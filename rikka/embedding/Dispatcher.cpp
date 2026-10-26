/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "Dispatcher.h"

#include "Bridge.h"

namespace mozilla::rikka {

Dispatcher::~Dispatcher() {
  if (RefPtr<Bridge> bridge = Bridge::Get()) {
    bridge->Forget(this);
  }
}

bool Dispatcher::HasEmbedderListener(const nsAString& aEvent) {
  RefPtr<Bridge> bridge = Bridge::Get();
  return bridge && bridge->HasEmbedderListener(mTarget, aEvent);
}

nsresult Dispatcher::DispatchToEmbedder(JSContext* aCx,
                                        const nsAString& aEvent,
                                        JS::Handle<JS::Value> aData,
                                        nsIGeckoViewEventCallback* aCallback) {
  RefPtr<Bridge> bridge = Bridge::Get();
  if (!bridge) {
    return NS_OK;
  }
  return bridge->SendEvent(aCx, mTarget, aEvent, aData, aCallback);
}

}  // namespace mozilla::rikka
