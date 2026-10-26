/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "Channel.h"

#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdlib.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <unistd.h>

#include "mozilla/EndianUtils.h"
#include "nsThreadUtils.h"

namespace mozilla::rikka {

static constexpr const char* kSocketVariable = "RIKKA_GECKO_CHANNEL";
static constexpr time_t kSendPatienceSeconds = 10;

static bool ReadExactly(int aSocket, char* aBuffer, size_t aLength) {
  while (aLength > 0) {
    ssize_t count = read(aSocket, aBuffer, aLength);
    if (count > 0) {
      aBuffer += count;
      aLength -= size_t(count);
    } else if (count < 0 && errno == EINTR) {
      continue;
    } else {
      return false;
    }
  }
  return true;
}

static bool WriteExactly(int aSocket, const char* aBuffer, size_t aLength) {
  while (aLength > 0) {
    ssize_t count = send(aSocket, aBuffer, aLength, MSG_NOSIGNAL);
    if (count > 0) {
      aBuffer += count;
      aLength -= size_t(count);
    } else if (count < 0 && errno == EINTR) {
      continue;
    } else {
      return false;
    }
  }
  return true;
}

already_AddRefed<Channel> Channel::FromEnvironment() {
  const char* value = getenv(kSocketVariable);
  if (!value || !*value) {
    return nullptr;
  }

  char* end = nullptr;
  errno = 0;
  long socket = strtol(value, &end, 10);
  if (errno || *end || socket <= STDERR_FILENO || socket > INT_MAX) {
    NS_WARNING("RIKKA_GECKO_CHANNEL does not name a file descriptor");
    return nullptr;
  }

  int flags = fcntl(int(socket), F_GETFD);
  if (flags < 0 || fcntl(int(socket), F_SETFD, flags | FD_CLOEXEC) < 0) {
    NS_WARNING("RIKKA_GECKO_CHANNEL is not an open file descriptor");
    return nullptr;
  }

  struct timeval patience = {kSendPatienceSeconds, 0};
  if (setsockopt(int(socket), SOL_SOCKET, SO_SNDTIMEO, &patience,
                 sizeof(patience)) < 0) {
    NS_WARNING("RIKKA_GECKO_CHANNEL is not a socket");
    return nullptr;
  }

  return do_AddRef(new Channel(UniqueFileHandle(int(socket))));
}

Channel::Channel(UniqueFileHandle aSocket) : mSocket(std::move(aSocket)) {}

nsresult Channel::Start(ChannelListener* aListener) {
  MOZ_ASSERT(NS_IsMainThread());
  mListener = aListener;

  nsresult rv = NS_NewNamedThread("RikkaChannelOut", getter_AddRefs(mWriter));
  NS_ENSURE_SUCCESS(rv, rv);

  RefPtr<Channel> self = this;
  return NS_NewNamedThread(
      "RikkaChannelIn", getter_AddRefs(mReader),
      NS_NewRunnableFunction("rikka::Channel::Read", [self] { self->Read(); }));
}

void Channel::Send(nsCString&& aFrame) {
  MOZ_ASSERT(NS_IsMainThread());
  if (!mWriter) {
    return;
  }
  if (aFrame.Length() > kMaxFrame) {
    NS_WARNING("Dropping a frame larger than the channel allows");
    return;
  }

  RefPtr<Channel> self = this;
  mWriter->Dispatch(NS_NewRunnableFunction(
      "rikka::Channel::Send", [self, frame = std::move(aFrame)] {
        char header[4];
        LittleEndian::writeUint32(header, frame.Length());
        int socket = self->mSocket.get();
        if (!WriteExactly(socket, header, sizeof(header)) ||
            !WriteExactly(socket, frame.BeginReading(), frame.Length())) {
          shutdown(socket, SHUT_RDWR);
        }
      }));
}

void Channel::Read() {
  int socket = mSocket.get();
  RefPtr<Channel> self = this;

  for (;;) {
    char header[4];
    if (!ReadExactly(socket, header, sizeof(header))) {
      break;
    }
    uint32_t length = LittleEndian::readUint32(header);
    if (length > kMaxFrame) {
      NS_WARNING("The embedder sent a frame larger than the channel allows");
      break;
    }
    nsCString frame;
    if (!frame.SetLength(length, fallible) ||
        !ReadExactly(socket, frame.BeginWriting(), length)) {
      break;
    }
    NS_DispatchToMainThread(NS_NewRunnableFunction(
        "rikka::Channel::Deliver", [self, frame = std::move(frame)]() mutable {
          self->Deliver(std::move(frame));
        }));
  }

  NS_DispatchToMainThread(
      NS_NewRunnableFunction("rikka::Channel::Close", [self] { self->Close(); }));
}

void Channel::Deliver(nsCString&& aFrame) {
  MOZ_ASSERT(NS_IsMainThread());
  if (mListener) {
    mListener->OnFrame(aFrame);
  }
}

void Channel::Close() {
  MOZ_ASSERT(NS_IsMainThread());
  if (ChannelListener* listener = std::exchange(mListener, nullptr)) {
    listener->OnClosed();
  }
}

void Channel::Shutdown() {
  MOZ_ASSERT(NS_IsMainThread());
  mListener = nullptr;
  if (nsCOMPtr<nsIThread> writer = std::move(mWriter)) {
    writer->Shutdown();
  }
  shutdown(mSocket.get(), SHUT_RDWR);
  if (nsCOMPtr<nsIThread> reader = std::move(mReader)) {
    reader->Shutdown();
  }
}

}  // namespace mozilla::rikka
