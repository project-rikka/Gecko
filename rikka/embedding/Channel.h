/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_rikka_Channel_h
#define mozilla_rikka_Channel_h

#include "mozilla/UniquePtrExtensions.h"
#include "nsCOMPtr.h"
#include "nsIThread.h"
#include "nsISupportsImpl.h"
#include "nsString.h"

namespace mozilla::rikka {

class ChannelListener {
 public:
  virtual void OnFrame(const nsACString& aFrame) = 0;
  virtual void OnClosed() = 0;

 protected:
  virtual ~ChannelListener() = default;
};

class Channel final {
 public:
  NS_INLINE_DECL_THREADSAFE_REFCOUNTING(Channel)

  static constexpr uint32_t kMaxFrame = 64 * 1024 * 1024;

  static already_AddRefed<Channel> FromEnvironment();

  nsresult Start(ChannelListener* aListener);
  void Send(nsCString&& aFrame);
  void Shutdown();

 private:
  explicit Channel(UniqueFileHandle aSocket);
  ~Channel() = default;

  void Read();
  void Deliver(nsCString&& aFrame);
  void Close();

  UniqueFileHandle mSocket;
  ChannelListener* mListener = nullptr;
  nsCOMPtr<nsIThread> mReader;
  nsCOMPtr<nsIThread> mWriter;
};

}  // namespace mozilla::rikka

#endif  // mozilla_rikka_Channel_h
