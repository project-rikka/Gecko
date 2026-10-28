/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "View.h"

#include "js/JSON.h"

namespace mozilla::rikka {

NS_IMPL_ISUPPORTS(View, nsIGeckoViewEventDispatcher, nsIGeckoViewView)

NS_IMETHODIMP
View::GetInitData(JSContext* aCx, JS::MutableHandle<JS::Value> aOut) {
  if (!JS_ParseJSON(aCx, mInitData.get(), mInitData.Length(), aOut)) {
    JS_ClearPendingException(aCx);
    return NS_ERROR_FAILURE;
  }
  return NS_OK;
}

}  // namespace mozilla::rikka
