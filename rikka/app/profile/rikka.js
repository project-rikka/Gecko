/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pref("app.update.enabled", false);
pref("browser.shell.checkDefaultBrowser", false);
pref("browser.startup.homepage_override.mstone", "ignore");
pref("toolkit.telemetry.enabled", false);
pref("toolkit.telemetry.unified", false);
pref("toolkit.telemetry.archive.enabled", false);
pref("datareporting.healthreport.uploadEnabled", false);
pref("datareporting.policy.dataSubmissionEnabled", false);
pref("app.shield.optoutstudies.enabled", false);
pref("app.normandy.enabled", false);
pref("geckoview.console.enabled", false);

pref("browser.link.open_newwindow", 3);
pref("browser.link.open_newwindow.restriction", 0);
pref("dom.disable_window_move_resize", true);
pref("dom.ipc.processPrelaunch.enabled", true);
pref("dom.ipc.keepProcessesAlive.web", 1);
pref("dom.ipc.keepProcessesAlive.extension", 1);
pref("dom.meta-viewport.enabled", false);
pref("browser.sessionhistory.max_entries", 50);
pref("browser.hangNotification.waitPeriod", 10000);
pref("network.android_doh.autoselect_enabled", false);
pref("ui.key.menuAccessKeyFocuses", false);

pref("gfx.webrender.compositor", false);
pref("gfx.webrender.all", true);
pref("media.hardware-video-decoding.enabled", true);
pref("media.hardware-video-decoding-vulkan.enabled", true);
pref("media.ffmpeg.vaapi.enabled", true);
pref("media.hevc.enabled", true);
pref("dom.webgpu.enabled", true);

pref("media.geckoview.autoplay.request", true);
pref("media.autoplay.default", 0);
pref("media.eme.require-app-approval", true);
pref("media.gmp-provider.enabled", true);
pref("media.gmp-gmpopenh264.visible", true);
pref("media.gmp-gmpopenh264.enabled", true);
#ifdef MOZ_WIDEVINE_EME
pref("media.gmp-manager.chromium-update-url", "https://update.googleapis.com/service/update2/crx?response=redirect&x=id%3D%GUID%%26uc&acceptformat=crx3&updaterversion=999");
pref("media.gmp-widevinecdm.visible", true);
pref("media.gmp-widevinecdm.enabled", true);
pref("media.gmp-widevinecdm.chromium-guid", "oimompecagnajdejgnnjijobebaeigek");
pref("media.gmp-widevinecdm.allow-chromium-update", true);
pref("media.gmp-widevinecdm.force-chromium-update", false);
pref("media.gmp-widevinecdm.force-chromium-beta", false);
#endif
pref("media.videocontrols.picture-in-picture.enabled", false);

pref("network.cookie.cookieBehavior", 5);
pref("network.cookie.cookieBehavior.pbmode", 5);
pref("privacy.trackingprotection.harmfuladdon.enabled", true);
pref("privacy.trackingprotection.fingerprinting.enabled", true);
pref("privacy.trackingprotection.cryptomining.enabled", true);
pref("privacy.trackingprotection.pbmode.enabled", true);
pref("browser.safebrowsing.malware.enabled", false);
pref("browser.safebrowsing.phishing.enabled", false);
pref("browser.safebrowsing.downloads.enabled", false);
pref("browser.safebrowsing.downloads.remote.enabled", false);

pref("browser.translations.enable", false);
pref("browser.translations.automaticallyPopup", false);
pref("pdfjs.handleOctetStream", true);
pref("browser.download.useDownloadDir", false);
pref("browser.download.always_ask_before_handling_new_types", true);
pref("browser.download.start_downloads_in_tmp_dir", false);
pref("layout.spellcheckDefault", 1);

pref("extensions.enabledScopes", 5);
pref("extensions.autoDisableScopes", 15);
pref("extensions.installDistroAddons", false);
pref("extensions.webextensions.remote", true);
pref("extensions.webextOptionalPermissionPrompts", true);
pref("extensions.update.enabled", true);
pref("extensions.update.autoUpdateDefault", true);
pref("extensions.update.interval", 86400);
pref("extensions.update.url", "https://versioncheck.addons.mozilla.org/update/VersionCheck.php?reqVersion=%REQ_VERSION%&id=%ITEM_ID%&version=%ITEM_VERSION%&maxAppVersion=%ITEM_MAXAPPVERSION%&status=%ITEM_STATUS%&appID=%APP_ID%&appVersion=%APP_VERSION%&appOS=%APP_OS%&appABI=%APP_ABI%&locale=%APP_LOCALE%&currentAppVersion=%CURRENT_APP_VERSION%&updateType=%UPDATE_TYPE%&compatMode=%COMPATIBILITY_MODE%");
pref("extensions.update.background.url", "https://versioncheck-bg.addons.mozilla.org/update/VersionCheck.php?reqVersion=%REQ_VERSION%&id=%ITEM_ID%&version=%ITEM_VERSION%&maxAppVersion=%ITEM_MAXAPPVERSION%&status=%ITEM_STATUS%&appID=%APP_ID%&appVersion=%APP_VERSION%&appOS=%APP_OS%&appABI=%APP_ABI%&locale=%APP_LOCALE%&currentAppVersion=%CURRENT_APP_VERSION%&updateType=%UPDATE_TYPE%&compatMode=%COMPATIBILITY_MODE%");
pref("extensions.getAddons.cache.enabled", true);
pref("extensions.getAddons.get.url", "https://services.addons.mozilla.org/api/v4/addons/search/?guid=%IDS%&lang=%LOCALE%");
pref("extensions.getAddons.search.browseURL", "https://addons.mozilla.org/%LOCALE%/firefox/search?q=%TERMS%&platform=%OS%&appver=%VERSION%");
pref("extensions.getAddons.link.url", "https://addons.mozilla.org/%LOCALE%/firefox/");
pref("extensions.getAddons.langpacks.url", "https://services.addons.mozilla.org/api/v4/addons/language-tools/?app=firefox&type=language&appversion=%VERSION%");
pref("xpinstall.signatures.required", true);
pref("xpinstall.whitelist.add", "https://addons.mozilla.org");

pref("devtools.chrome.enabled", false);
pref("devtools.debugger.remote-enabled", false);
