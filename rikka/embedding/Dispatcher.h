/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_rikka_Dispatcher_h
#define mozilla_rikka_Dispatcher_h

#include "mozilla/widget/EventDispatcherBase.h"
#include "nsString.h"

namespace mozilla::rikka {

class Dispatcher final : public widget::EventDispatcherBase {
 public:
  explicit Dispatcher(const nsACString& aTarget) : mTarget(aTarget) {}

  const nsCString& Target() const { return mTarget; }

  bool HasEmbedderListener(const nsAString& aEvent) override
      MOZ_REQUIRES(sMainThreadCapability);
  nsresult DispatchToEmbedder(JSContext* aCx, const nsAString& aEvent,
                              JS::Handle<JS::Value> aData,
                              nsIGeckoViewEventCallback* aCallback) override
      MOZ_REQUIRES(sMainThreadCapability);

 private:
  ~Dispatcher() override;

  const nsCString mTarget;
};

}  // namespace mozilla::rikka

#endif  // mozilla_rikka_Dispatcher_h
