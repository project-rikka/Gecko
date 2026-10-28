/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>
#include <strings.h>
#include <unistd.h>

#include "ApplicationData.h"
#include "BinaryPath.h"
#include "XREChildData.h"
#include "mozilla/BaseProfiler.h"
#include "mozilla/Bootstrap.h"
#include "mozilla/ProcessType.h"
#include "mozilla/RuntimeExceptionModule.h"
#include "mozilla/ScopeExit.h"
#include "mozilla/StartupTimeline.h"
#include "mozilla/UniquePtrExtensions.h"
#include "mozilla/XREAppData.h"
#include "nsXULAppAPI.h"

using namespace mozilla;

static MOZ_FORMAT_PRINTF(1, 2) void Output(const char* aFormat, ...) {
  va_list arguments;
  va_start(arguments, aFormat);
  vfprintf(stderr, aFormat, arguments);
  va_end(arguments);
}

static bool IsFlag(const char* aArgument, const char* aFlag) {
  if (*aArgument != '-') {
    return false;
  }
  ++aArgument;
  if (*aArgument == '-') {
    ++aArgument;
  }
  return !strcasecmp(aArgument, aFlag);
}

constinit Bootstrap::UniquePtr gBootstrap;

static nsresult InitXPCOMGlue(LibLoadingStrategy aStrategy) {
  if (gBootstrap) {
    return NS_OK;
  }

  UniqueFreePtr<char> executable = BinaryPath::Get();
  if (!executable) {
    Output("Couldn't find the application directory.\n");
    return NS_ERROR_FAILURE;
  }

  auto bootstrap = mozilla::GetBootstrap(executable.get(), aStrategy);
  if (bootstrap.isErr()) {
    Output("Couldn't load XPCOM.\n");
    return NS_ERROR_FAILURE;
  }

  gBootstrap = bootstrap.unwrap();
  gBootstrap->NS_LogInit();
  return NS_OK;
}

static void ReserveDefaultFileDescriptors() {
  int null = open("/dev/null", O_RDONLY);
  for (int i = 0; i < 2; i++) {
    [[maybe_unused]] int duplicate = dup(null);
  }
}

static void ExpandFileDescriptorTable() {
  mozilla::UniqueFileHandle expander(fcntl(0, F_DUPFD, 256));
}

static int RunChildProcess(int aArgc, char* aArgv[]) {
  nsresult rv = InitXPCOMGlue(LibLoadingStrategy::NoReadAhead);
  if (NS_FAILED(rv)) {
    return 255;
  }

  XREChildData childData;
  rv = gBootstrap->XRE_InitChildProcess(aArgc, aArgv, &childData);
  gBootstrap->NS_LogTerm();
  return NS_FAILED(rv) ? 1 : 0;
}

int main(int argc, char* argv[]) {
  ReserveDefaultFileDescriptors();

  if (argc > 1 && IsFlag(argv[1], "contentproc")) {
    SetGeckoProcessType(argv[--argc]);
    SetGeckoChildID(argv[--argc]);

#if defined(MOZ_ENABLE_FORKSERVER)
    if (GetGeckoProcessType() == GeckoProcessType_ForkServer) {
      if (NS_FAILED(InitXPCOMGlue(LibLoadingStrategy::NoReadAhead))) {
        return 255;
      }
      if (gBootstrap->XRE_ForkServer(&argc, &argv)) {
        gBootstrap->NS_LogTerm();
        return 0;
      }
    }
#endif
  }

  ExpandFileDescriptorTable();

  mozilla::TimeStamp start = mozilla::TimeStamp::Now();

  AUTO_BASE_PROFILER_INIT;
  AUTO_BASE_PROFILER_LABEL("nsRikkaApp main", OTHER);

  CrashReporter::RegisterRuntimeExceptionModule();
  auto unregisterRuntimeExceptionModule =
      MakeScopeExit([] { CrashReporter::UnregisterRuntimeExceptionModule(); });

  if (GetGeckoProcessType() != GeckoProcessType_Default) {
    return RunChildProcess(argc, argv);
  }

  if (NS_FAILED(InitXPCOMGlue(LibLoadingStrategy::ReadAhead))) {
    return 255;
  }

  gBootstrap->XRE_StartupTimelineRecord(mozilla::StartupTimeline::START, start);
  gBootstrap->XRE_EnableSameExecutableForContentProc();

  BootstrapConfig config;
  config.appData = kStaticAppData;
  config.appDataPath = nullptr;

  int result = gBootstrap->XRE_main(argc, argv, config);

  gBootstrap->NS_LogTerm();
  gBootstrap.reset();
  return result;
}
