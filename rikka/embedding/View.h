/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_rikka_View_h
#define mozilla_rikka_View_h

#include "Dispatcher.h"
#include "nsIGeckoViewBridge.h"
#include "nsString.h"

namespace mozilla::rikka {

class View final : public nsIGeckoViewView {
 public:
  NS_DECL_ISUPPORTS
  NS_DECL_NSIGECKOVIEWVIEW
  NS_FORWARD_NSIGECKOVIEWEVENTDISPATCHER(mDispatcher->)

  View(Dispatcher* aDispatcher, const nsAString& aInitData)
      : mDispatcher(aDispatcher), mInitData(aInitData) {}

  Dispatcher* GetDispatcher() const { return mDispatcher; }

 private:
  ~View() = default;

  const RefPtr<Dispatcher> mDispatcher;
  const nsString mInitData;
};

}  // namespace mozilla::rikka

#endif  // mozilla_rikka_View_h
