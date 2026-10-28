/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#ifndef mozilla_rikka_Bridge_h
#define mozilla_rikka_Bridge_h

#include "Channel.h"
#include "Dispatcher.h"
#include "View.h"
#include "nsCOMPtr.h"
#include "nsIGeckoViewBridge.h"
#include "nsIObserver.h"
#include "nsPIDOMWindow.h"
#include "nsTHashMap.h"
#include "nsTHashSet.h"

class nsIWidget;
class mozIDOMWindowProxy;

namespace mozilla::rikka {

class Bridge final : public nsIGeckoViewBridge,
                     public nsIObserver,
                     public ChannelListener {
 public:
  NS_DECL_ISUPPORTS
  NS_DECL_NSIGECKOVIEWBRIDGE
  NS_DECL_NSIOBSERVER
  NS_FORWARD_NSIGECKOVIEWEVENTDISPATCHER(mRuntime->)

  static constexpr uint32_t kProtocol = 1;

  static already_AddRefed<Bridge> GetSingleton();
  static already_AddRefed<Bridge> Get();

  bool HasEmbedderListener(const nsACString& aTarget,
                           const nsAString& aEvent) const;
  nsresult SendEvent(JSContext* aCx, const nsACString& aTarget,
                     const nsAString& aEvent, JS::Handle<JS::Value> aData,
                     nsIGeckoViewEventCallback* aCallback);
  void SendReply(JSContext* aCx, uint64_t aCallback, bool aSuccess,
                 JS::Handle<JS::Value> aData);
  void SendRelease(uint64_t aCallback);
  void Forget(Dispatcher* aDispatcher);
  widget::EventDispatcherBase* DispatcherForWidget(nsIWidget* aWidget) const;

  void OnFrame(const nsACString& aFrame) override;
  void OnClosed() override;

 private:
  struct Window {
    RefPtr<View> mView;
    nsCOMPtr<nsPIDOMWindowOuter> mOuter;
    nsCOMPtr<nsIWidget> mWidget;
  };

  Bridge() = default;
  ~Bridge() override = default;

  void Init();
  void Shutdown();
  void Send(JSContext* aCx, JS::Handle<JSObject*> aMessage);
  void SendHello();
  void SendWindowMessage(const char* aType, const nsAString& aWindow);
  void Handle(JSContext* aCx, JS::Handle<JSObject*> aMessage);
  void HandleListen(JSContext* aCx, JS::Handle<JSObject*> aMessage,
                    bool aListen);
  void HandleEvent(JSContext* aCx, JS::Handle<JSObject*> aMessage);
  void HandleReply(JSContext* aCx, JS::Handle<JSObject*> aMessage);
  void HandleRelease(JSContext* aCx, JS::Handle<JSObject*> aMessage);
  void HandleOpen(JSContext* aCx, JS::Handle<JSObject*> aMessage);
  void HandleClose(JSContext* aCx, JS::Handle<JSObject*> aMessage);
  Dispatcher* Target(const nsACString& aTarget) const;
  void OpenWindow(const nsAString& aWindow, const nsAString& aInitData,
                  bool aPrivate);
  void CloseWindow(const nsAString& aWindow);
  void WindowClosed(mozIDOMWindowProxy* aWindow);
  void WindowLoaded(nsPIDOMWindowOuter* aOuter);
  void Quit();

  RefPtr<Channel> mChannel;
  RefPtr<Dispatcher> mRuntime;
  nsTHashMap<nsCStringHashKey, Dispatcher*> mNamed;
  nsTHashMap<nsStringHashKey, Window> mWindows;
  nsTHashMap<nsCStringHashKey, nsTHashSet<nsString>> mListeners;
  nsTHashMap<nsUint64HashKey, nsCOMPtr<nsIGeckoViewEventCallback>> mPending;
  uint64_t mNextCallback = 1;
};

}  // namespace mozilla::rikka

#endif  // mozilla_rikka_Bridge_h
