/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "Bridge.h"

#include "js/Array.h"
#include "js/JSON.h"
#include "js/PropertyAndElement.h"
#include "mozilla/ClearOnShutdown.h"
#include "mozilla/Components.h"
#include "mozilla/Preferences.h"
#include "mozilla/Services.h"
#include "mozilla/StaticPtr.h"
#include "mozilla/dom/ScriptSettings.h"
#include "mozilla/dom/Document.h"
#include "mozilla/dom/ToJSValue.h"
#include "mozilla/WidgetUtils.h"
#include "nsContentUtils.h"
#include "nsIAppStartup.h"
#include "nsIAppWindow.h"
#include "nsIObserverService.h"
#include "nsIWidget.h"
#include "nsIWidgetListener.h"
#include "nsIWindowWatcher.h"
#include "nsIXULAppInfo.h"
#include "nsJSUtils.h"
#include "nsPrintfCString.h"
#include "nsServiceManagerUtils.h"
#include "xpcpublic.h"

namespace mozilla::rikka {

namespace {

constexpr auto kRuntimeTarget = "runtime"_ns;
constexpr auto kNamedPrefix = "named:"_ns;
constexpr auto kWindowPrefix = "window:"_ns;
constexpr auto kDefaultChromeURI = "chrome://rikka/content/view.xhtml"_ns;
constexpr double kMaxCallbackId = 9007199254740991.0;

StaticRefPtr<Bridge> sBridge;

bool GetValue(JSContext* aCx, JS::Handle<JSObject*> aObject, const char* aName,
              JS::MutableHandle<JS::Value> aValue) {
  if (!JS_GetProperty(aCx, aObject, aName, aValue)) {
    JS_ClearPendingException(aCx);
    return false;
  }
  return true;
}

bool GetString(JSContext* aCx, JS::Handle<JSObject*> aObject, const char* aName,
               nsAString& aString) {
  JS::Rooted<JS::Value> value(aCx);
  if (!GetValue(aCx, aObject, aName, &value) || !value.isString()) {
    return false;
  }
  nsAutoJSString string;
  if (!string.init(aCx, value.toString())) {
    JS_ClearPendingException(aCx);
    return false;
  }
  aString.Assign(string);
  return true;
}

bool GetUTF8String(JSContext* aCx, JS::Handle<JSObject*> aObject,
                   const char* aName, nsACString& aString) {
  nsAutoString string;
  if (!GetString(aCx, aObject, aName, string)) {
    return false;
  }
  CopyUTF16toUTF8(string, aString);
  return true;
}

Maybe<uint64_t> GetCallbackId(JSContext* aCx, JS::Handle<JSObject*> aObject) {
  JS::Rooted<JS::Value> value(aCx);
  if (!GetValue(aCx, aObject, "callback", &value) || !value.isNumber()) {
    return Nothing();
  }
  double number = value.toNumber();
  if (!(number >= 1.0 && number <= kMaxCallbackId) ||
      number != double(uint64_t(number))) {
    return Nothing();
  }
  return Some(uint64_t(number));
}

bool GetBool(JSContext* aCx, JS::Handle<JSObject*> aObject, const char* aName) {
  JS::Rooted<JS::Value> value(aCx);
  return GetValue(aCx, aObject, aName, &value) && value.isBoolean() &&
         value.toBoolean();
}

bool GetStrings(JSContext* aCx, JS::Handle<JSObject*> aObject,
                const char* aName, nsTArray<nsString>& aStrings) {
  JS::Rooted<JS::Value> value(aCx);
  if (!GetValue(aCx, aObject, aName, &value) || !value.isObject()) {
    return false;
  }
  JS::Rooted<JSObject*> array(aCx, &value.toObject());
  bool isArray = false;
  uint32_t length = 0;
  if (!JS::IsArrayObject(aCx, array, &isArray) || !isArray ||
      !JS::GetArrayLength(aCx, array, &length)) {
    JS_ClearPendingException(aCx);
    return false;
  }
  for (uint32_t index = 0; index < length; index++) {
    JS::Rooted<JS::Value> element(aCx);
    if (!JS_GetElement(aCx, array, index, &element) || !element.isString()) {
      JS_ClearPendingException(aCx);
      return false;
    }
    nsAutoJSString string;
    if (!string.init(aCx, element.toString())) {
      JS_ClearPendingException(aCx);
      return false;
    }
    aStrings.AppendElement(string);
  }
  return true;
}

template <typename T>
bool SetValue(JSContext* aCx, JS::Handle<JSObject*> aObject, const char* aName,
              T&& aValue) {
  JS::Rooted<JS::Value> value(aCx);
  if (!dom::ToJSValue(aCx, std::forward<T>(aValue), &value) ||
      !JS_DefineProperty(aCx, aObject, aName, value, JSPROP_ENUMERATE)) {
    JS_ClearPendingException(aCx);
    return false;
  }
  return true;
}

bool SetJSValue(JSContext* aCx, JS::Handle<JSObject*> aObject,
                const char* aName, JS::Handle<JS::Value> aValue) {
  if (!JS_DefineProperty(aCx, aObject, aName, aValue, JSPROP_ENUMERATE)) {
    JS_ClearPendingException(aCx);
    return false;
  }
  return true;
}

JSObject* NewMessage(JSContext* aCx, const char* aType) {
  JS::Rooted<JSObject*> message(aCx, JS_NewPlainObject(aCx));
  if (!message || !SetValue(aCx, message, "type", NS_ConvertASCIItoUTF16(aType))) {
    JS_ClearPendingException(aCx);
    return nullptr;
  }
  return message;
}

class EmbedderCallback final : public nsIGeckoViewEventCallback {
 public:
  NS_DECL_ISUPPORTS
  NS_DECL_NSIGECKOVIEWEVENTCALLBACK

  explicit EmbedderCallback(uint64_t aId) : mId(aId) {}

 private:
  ~EmbedderCallback() {
    if (mAnswered) {
      return;
    }
    if (RefPtr<Bridge> bridge = Bridge::Get()) {
      bridge->SendRelease(mId);
    }
  }

  nsresult Answer(JSContext* aCx, bool aSuccess, JS::Handle<JS::Value> aData) {
    if (std::exchange(mAnswered, true)) {
      return NS_OK;
    }
    if (RefPtr<Bridge> bridge = Bridge::Get()) {
      bridge->SendReply(aCx, mId, aSuccess, aData);
    }
    return NS_OK;
  }

  const uint64_t mId;
  bool mAnswered = false;
};

NS_IMPL_ISUPPORTS(EmbedderCallback, nsIGeckoViewEventCallback)

NS_IMETHODIMP
EmbedderCallback::OnSuccess(JS::Handle<JS::Value> aData, JSContext* aCx) {
  return Answer(aCx, true, aData);
}

NS_IMETHODIMP
EmbedderCallback::OnError(JS::Handle<JS::Value> aData, JSContext* aCx) {
  return Answer(aCx, false, aData);
}

}  // namespace

NS_IMPL_ISUPPORTS(Bridge, nsIGeckoViewEventDispatcher, nsIGeckoViewBridge,
                  nsIObserver)

already_AddRefed<Bridge> Bridge::GetSingleton() {
  MOZ_ASSERT(NS_IsMainThread());
  if (!sBridge) {
    sBridge = new Bridge();
    sBridge->Init();
    ClearOnShutdown(&sBridge);
  }
  return do_AddRef(sBridge);
}

already_AddRefed<Bridge> Bridge::Get() {
  MOZ_ASSERT(NS_IsMainThread());
  return do_AddRef(sBridge);
}

void Bridge::Init() {
  mRuntime = new Dispatcher(kRuntimeTarget);

  if (nsCOMPtr<nsIObserverService> observers = services::GetObserverService()) {
    observers->AddObserver(this, "xpcom-will-shutdown", false);
    observers->AddObserver(this, "chrome-document-loaded", false);
  }
  if (nsCOMPtr<nsIWindowWatcher> watcher =
          do_GetService(NS_WINDOWWATCHER_CONTRACTID)) {
    watcher->RegisterNotification(this);
  }

  mChannel = Channel::FromEnvironment();
  if (!mChannel) {
    return;
  }
  if (NS_FAILED(mChannel->Start(this))) {
    mChannel = nullptr;
    return;
  }
  if (nsCOMPtr<nsIAppStartup> startup = components::AppStartup::Service()) {
    startup->EnterLastWindowClosingSurvivalArea();
  }
  SendHello();
}

void Bridge::Shutdown() {
  if (RefPtr<Channel> channel = std::move(mChannel)) {
    channel->Shutdown();
  }
  mPending.Clear();
  mListeners.Clear();
  mWindows.Clear();
  mNamed.Clear();
  if (nsCOMPtr<nsIWindowWatcher> watcher =
          do_GetService(NS_WINDOWWATCHER_CONTRACTID)) {
    watcher->UnregisterNotification(this);
  }
}

NS_IMETHODIMP
Bridge::Observe(nsISupports* aSubject, const char* aTopic,
                const char16_t* aData) {
  if (!strcmp(aTopic, "domwindowclosed")) {
    nsCOMPtr<mozIDOMWindowProxy> window = do_QueryInterface(aSubject);
    WindowClosed(window);
  } else if (!strcmp(aTopic, "chrome-document-loaded")) {
    if (nsCOMPtr<dom::Document> document = do_QueryInterface(aSubject)) {
      WindowLoaded(document->GetWindow());
    }
  } else if (!strcmp(aTopic, "xpcom-will-shutdown")) {
    if (nsCOMPtr<nsIObserverService> observers =
            services::GetObserverService()) {
      observers->RemoveObserver(this, "xpcom-will-shutdown");
      observers->RemoveObserver(this, "chrome-document-loaded");
    }
    Shutdown();
  }
  return NS_OK;
}

NS_IMETHODIMP
Bridge::GetDispatcherByName(const char* aName,
                            nsIGeckoViewEventDispatcher** aResult) {
  AssertIsOnMainThread();
  nsAutoCString target(kNamedPrefix);
  target.Append(aName);

  RefPtr<Dispatcher> dispatcher = mNamed.Get(target);
  if (!dispatcher) {
    dispatcher = new Dispatcher(target);
    mNamed.InsertOrUpdate(target, dispatcher.get());
  }
  dispatcher.forget(aResult);
  return NS_OK;
}

void Bridge::Forget(Dispatcher* aDispatcher) {
  const nsCString& target = aDispatcher->Target();
  if (mNamed.Get(target) == aDispatcher) {
    mNamed.Remove(target);
    mListeners.Remove(target);
  }
}

bool Bridge::HasEmbedderListener(const nsACString& aTarget,
                                 const nsAString& aEvent) const {
  const nsTHashSet<nsString>* events = mListeners.GetValue(aTarget);
  return mChannel && events && events->Contains(aEvent);
}

nsresult Bridge::SendEvent(JSContext* aCx, const nsACString& aTarget,
                           const nsAString& aEvent,
                           JS::Handle<JS::Value> aData,
                           nsIGeckoViewEventCallback* aCallback) {
  AssertIsOnMainThread();
  if (!HasEmbedderListener(aTarget, aEvent)) {
    if (aCallback) {
      nsAutoString reason(u"No listener for "_ns);
      reason.Append(aEvent);
      JS::Rooted<JS::Value> error(aCx);
      if (!dom::ToJSValue(aCx, reason, &error)) {
        JS_ClearPendingException(aCx);
        return NS_ERROR_OUT_OF_MEMORY;
      }
      return aCallback->OnError(error, aCx);
    }
    return NS_OK;
  }

  JS::Rooted<JSObject*> message(aCx, NewMessage(aCx, "event"));
  if (!message || !SetValue(aCx, message, "target", aTarget) ||
      !SetValue(aCx, message, "event", aEvent) ||
      !SetJSValue(aCx, message, "data", aData)) {
    return NS_ERROR_FAILURE;
  }

  if (aCallback) {
    uint64_t id = mNextCallback++;
    if (!SetValue(aCx, message, "callback", double(id))) {
      return NS_ERROR_FAILURE;
    }
    mPending.InsertOrUpdate(id, aCallback);
  }

  Send(aCx, message);
  return NS_OK;
}

void Bridge::SendReply(JSContext* aCx, uint64_t aCallback, bool aSuccess,
                       JS::Handle<JS::Value> aData) {
  JS::Rooted<JSObject*> message(aCx, NewMessage(aCx, "reply"));
  if (!message || !SetValue(aCx, message, "callback", double(aCallback)) ||
      !SetValue(aCx, message, "success", aSuccess) ||
      !SetJSValue(aCx, message, "data", aData)) {
    return;
  }
  Send(aCx, message);
}

void Bridge::SendRelease(uint64_t aCallback) {
  if (!mChannel) {
    return;
  }
  dom::AutoJSAPI jsapi;
  if (!jsapi.Init(xpc::PrivilegedJunkScope())) {
    return;
  }
  JSContext* cx = jsapi.cx();
  JS::Rooted<JSObject*> message(cx, NewMessage(cx, "release"));
  if (!message || !SetValue(cx, message, "callback", double(aCallback))) {
    return;
  }
  Send(cx, message);
}

void Bridge::SendHello() {
  dom::AutoJSAPI jsapi;
  if (!jsapi.Init(xpc::PrivilegedJunkScope())) {
    return;
  }
  JSContext* cx = jsapi.cx();

  nsAutoCString version;
  if (nsCOMPtr<nsIXULAppInfo> appInfo =
          do_GetService("@mozilla.org/xre/app-info;1")) {
    appInfo->GetVersion(version);
  }

  JS::Rooted<JSObject*> message(cx, NewMessage(cx, "hello"));
  if (!message || !SetValue(cx, message, "protocol", kProtocol) ||
      !SetValue(cx, message, "version", version)) {
    return;
  }
  Send(cx, message);
}

void Bridge::SendWindowMessage(const char* aType, const nsAString& aWindow) {
  if (!mChannel) {
    return;
  }
  dom::AutoJSAPI jsapi;
  if (!jsapi.Init(xpc::PrivilegedJunkScope())) {
    return;
  }
  JSContext* cx = jsapi.cx();
  JS::Rooted<JSObject*> message(cx, NewMessage(cx, aType));
  if (!message || !SetValue(cx, message, "window", aWindow)) {
    return;
  }
  Send(cx, message);
}

void Bridge::Send(JSContext* aCx, JS::Handle<JSObject*> aMessage) {
  if (!mChannel) {
    return;
  }
  JS::Rooted<JS::Value> value(aCx, JS::ObjectValue(*aMessage));
  nsAutoString json;
  if (!nsContentUtils::StringifyJSON(aCx, value, json,
                                     UndefinedIsNullStringLiteral)) {
    JS_ClearPendingException(aCx);
    NS_WARNING("Dropping a message that cannot be serialized as JSON");
    return;
  }
  mChannel->Send(NS_ConvertUTF16toUTF8(json));
}

void Bridge::OnFrame(const nsACString& aFrame) {
  AssertIsOnMainThread();
  dom::AutoJSAPI jsapi;
  if (!jsapi.Init(xpc::PrivilegedJunkScope())) {
    return;
  }
  JSContext* cx = jsapi.cx();

  NS_ConvertUTF8toUTF16 text(aFrame);
  JS::Rooted<JS::Value> value(cx);
  if (!JS_ParseJSON(cx, text.get(), text.Length(), &value) ||
      !value.isObject()) {
    JS_ClearPendingException(cx);
    NS_WARNING("The embedder sent a frame that is not a JSON object");
    return;
  }
  JS::Rooted<JSObject*> message(cx, &value.toObject());
  Handle(cx, message);
}

void Bridge::Handle(JSContext* aCx, JS::Handle<JSObject*> aMessage) {
  nsAutoCString type;
  if (!GetUTF8String(aCx, aMessage, "type", type)) {
    NS_WARNING("The embedder sent a message without a type");
    return;
  }

  if (type.EqualsLiteral("event")) {
    HandleEvent(aCx, aMessage);
  } else if (type.EqualsLiteral("reply")) {
    HandleReply(aCx, aMessage);
  } else if (type.EqualsLiteral("release")) {
    HandleRelease(aCx, aMessage);
  } else if (type.EqualsLiteral("listen")) {
    HandleListen(aCx, aMessage, true);
  } else if (type.EqualsLiteral("unlisten")) {
    HandleListen(aCx, aMessage, false);
  } else if (type.EqualsLiteral("open")) {
    HandleOpen(aCx, aMessage);
  } else if (type.EqualsLiteral("close")) {
    HandleClose(aCx, aMessage);
  } else {
    NS_WARNING(nsPrintfCString("Unknown embedder message %s", type.get()).get());
  }
}

void Bridge::HandleListen(JSContext* aCx, JS::Handle<JSObject*> aMessage,
                          bool aListen) {
  nsAutoCString target;
  nsTArray<nsString> events;
  if (!GetUTF8String(aCx, aMessage, "target", target) ||
      !GetStrings(aCx, aMessage, "events", events)) {
    NS_WARNING("Malformed listen message");
    return;
  }

  if (aListen) {
    nsTHashSet<nsString>& listened = mListeners.LookupOrInsert(target);
    for (const nsString& event : events) {
      listened.Insert(event);
    }
    return;
  }

  if (auto listened = mListeners.Lookup(target)) {
    for (const nsString& event : events) {
      listened->Remove(event);
    }
    if (listened->IsEmpty()) {
      listened.Remove();
    }
  }
}

void Bridge::HandleEvent(JSContext* aCx, JS::Handle<JSObject*> aMessage) {
  AssertIsOnMainThread();
  nsAutoCString target;
  nsAutoString event;
  if (!GetUTF8String(aCx, aMessage, "target", target) ||
      !GetString(aCx, aMessage, "event", event)) {
    NS_WARNING("Malformed event message");
    return;
  }
  JS::Rooted<JS::Value> data(aCx);
  if (!GetValue(aCx, aMessage, "data", &data)) {
    return;
  }

  nsCOMPtr<nsIGeckoViewEventCallback> callback;
  if (Maybe<uint64_t> id = GetCallbackId(aCx, aMessage)) {
    callback = new EmbedderCallback(*id);
  }

  RefPtr<Dispatcher> dispatcher = Target(target);
  if (!dispatcher) {
    if (callback) {
      JS::Rooted<JS::Value> error(aCx);
      if (dom::ToJSValue(aCx, u"No such target"_ns, &error)) {
        callback->OnError(error, aCx);
      }
    }
    return;
  }
  dispatcher->DispatchToGecko(aCx, event, data, callback);
}

void Bridge::HandleReply(JSContext* aCx, JS::Handle<JSObject*> aMessage) {
  Maybe<uint64_t> id = GetCallbackId(aCx, aMessage);
  if (!id) {
    NS_WARNING("Malformed reply message");
    return;
  }
  Maybe<nsCOMPtr<nsIGeckoViewEventCallback>> callback = mPending.Extract(*id);
  if (!callback) {
    return;
  }
  JS::Rooted<JS::Value> data(aCx);
  if (!GetValue(aCx, aMessage, "data", &data)) {
    return;
  }
  if (GetBool(aCx, aMessage, "success")) {
    (*callback)->OnSuccess(data, aCx);
  } else {
    (*callback)->OnError(data, aCx);
  }
}

void Bridge::HandleRelease(JSContext* aCx, JS::Handle<JSObject*> aMessage) {
  if (Maybe<uint64_t> id = GetCallbackId(aCx, aMessage)) {
    mPending.Remove(*id);
  }
}

void Bridge::HandleOpen(JSContext* aCx, JS::Handle<JSObject*> aMessage) {
  nsAutoString window;
  JS::Rooted<JS::Value> init(aCx);
  if (!GetString(aCx, aMessage, "window", window) || window.IsEmpty() ||
      !GetValue(aCx, aMessage, "init", &init) || !init.isObject()) {
    NS_WARNING("Malformed open message");
    return;
  }
  nsAutoString initData;
  if (!nsContentUtils::StringifyJSON(aCx, init, initData,
                                     UndefinedIsNullStringLiteral)) {
    JS_ClearPendingException(aCx);
    return;
  }
  OpenWindow(window, initData, GetBool(aCx, aMessage, "private"));
}

void Bridge::HandleClose(JSContext* aCx, JS::Handle<JSObject*> aMessage) {
  nsAutoString window;
  if (!GetString(aCx, aMessage, "window", window)) {
    NS_WARNING("Malformed close message");
    return;
  }
  CloseWindow(window);
}

Dispatcher* Bridge::Target(const nsACString& aTarget) const {
  if (aTarget.Equals(kRuntimeTarget)) {
    return mRuntime;
  }
  if (StringBeginsWith(aTarget, kNamedPrefix)) {
    return mNamed.Get(aTarget);
  }
  if (StringBeginsWith(aTarget, kWindowPrefix)) {
    NS_ConvertUTF8toUTF16 window(Substring(aTarget, kWindowPrefix.Length()));
    const Window* entry = mWindows.GetValue(window);
    return entry ? entry->mView->GetDispatcher() : nullptr;
  }
  return nullptr;
}

void Bridge::OpenWindow(const nsAString& aWindow, const nsAString& aInitData,
                        bool aPrivate) {
  if (mWindows.Contains(aWindow)) {
    NS_WARNING("The embedder opened a window id twice");
    return;
  }

  nsCOMPtr<nsIWindowWatcher> watcher =
      do_GetService(NS_WINDOWWATCHER_CONTRACTID);
  if (!watcher) {
    SendWindowMessage("closed", aWindow);
    return;
  }

  nsAutoCString url;
  if (NS_FAILED(Preferences::GetCString("toolkit.defaultChromeURI", url))) {
    url = kDefaultChromeURI;
  }

  nsAutoCString target(kWindowPrefix);
  AppendUTF16toUTF8(aWindow, target);
  RefPtr<View> view = new View(new Dispatcher(target), aInitData);

  nsAutoCString features("chrome,dialog=0,remote,resizable");
  if (aPrivate) {
    features.AppendLiteral(",private");
  }

  nsCOMPtr<mozIDOMWindowProxy> domWindow;
  nsresult rv = watcher->OpenWindow(nullptr, url, aWindow, features, view,
                                    getter_AddRefs(domWindow));
  if (NS_FAILED(rv) || !domWindow) {
    SendWindowMessage("closed", aWindow);
    return;
  }

  nsCOMPtr<nsPIDOMWindowOuter> outer = nsPIDOMWindowOuter::From(domWindow);
  nsCOMPtr<nsIWidget> widget = WidgetUtils::DOMWindowToWidget(outer);
  if (widget) {
    if (nsIWidgetListener* listener = widget->GetWidgetListener()) {
      if (nsCOMPtr<nsIAppWindow> appWindow = listener->GetAppWindow()) {
        appWindow->SetIntrinsicallySized(false);
      }
    }
  }

  mWindows.InsertOrUpdate(aWindow, Window{view, outer, widget});
}

void Bridge::CloseWindow(const nsAString& aWindow) {
  Maybe<Window> window = mWindows.Extract(aWindow);
  if (!window) {
    return;
  }
  nsAutoCString target(kWindowPrefix);
  AppendUTF16toUTF8(aWindow, target);
  mListeners.Remove(target);
  window->mOuter->ForceClose();
}

void Bridge::WindowClosed(mozIDOMWindowProxy* aWindow) {
  nsCOMPtr<nsPIDOMWindowOuter> outer = nsPIDOMWindowOuter::From(aWindow);
  if (!outer) {
    return;
  }
  for (auto iter = mWindows.Iter(); !iter.Done(); iter.Next()) {
    if (iter.Data().mOuter != outer) {
      continue;
    }
    nsString window(iter.Key());
    iter.Remove();
    nsAutoCString target(kWindowPrefix);
    AppendUTF16toUTF8(window, target);
    mListeners.Remove(target);
    SendWindowMessage("closed", window);
    return;
  }
}

void Bridge::WindowLoaded(nsPIDOMWindowOuter* aOuter) {
  if (!aOuter) {
    return;
  }
  for (const auto& entry : mWindows) {
    if (entry.GetData().mOuter == aOuter) {
      SendWindowMessage("ready", entry.GetKey());
      return;
    }
  }
}

widget::EventDispatcherBase* Bridge::DispatcherForWidget(
    nsIWidget* aWidget) const {
  if (!aWidget) {
    return nullptr;
  }
  nsIWidget* topLevel = aWidget->GetTopLevelWidget();
  for (const auto& entry : mWindows.Values()) {
    if (entry.mWidget == aWidget || entry.mWidget == topLevel) {
      return entry.mView->GetDispatcher();
    }
  }
  return nullptr;
}

void Bridge::OnClosed() {
  AssertIsOnMainThread();
  if (RefPtr<Channel> channel = std::move(mChannel)) {
    channel->Shutdown();
  }
  mPending.Clear();
  Quit();
}

void Bridge::Quit() {
  nsTArray<nsCOMPtr<nsPIDOMWindowOuter>> windows;
  for (const auto& entry : mWindows.Values()) {
    windows.AppendElement(entry.mOuter);
  }
  mWindows.Clear();
  mListeners.Clear();
  for (const auto& window : windows) {
    window->ForceClose();
  }

  if (nsCOMPtr<nsIAppStartup> startup = components::AppStartup::Service()) {
    bool userAllowedQuit = false;
    startup->Quit(nsIAppStartup::eForceQuit, 0, &userAllowedQuit);
  }
}

}  // namespace mozilla::rikka

namespace mozilla::widget {

EventDispatcherBase* GeckoViewDispatcherForWidget(nsIWidget* aWidget) {
  RefPtr<rikka::Bridge> bridge = rikka::Bridge::Get();
  return bridge ? bridge->DispatcherForWidget(aWidget) : nullptr;
}

}  // namespace mozilla::widget
