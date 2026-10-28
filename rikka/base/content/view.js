/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

"use strict";

document.documentElement.setAttribute("windowclass", `rikka-view-${window.name}`);

for (const [module, name] of [
  ["resource://gre/modules/RikkaUserContent.sys.mjs", "RikkaUserContent"],
  ["resource://gre/modules/RikkaView.sys.mjs", "RikkaView"],
  ["resource://gre/modules/RikkaEditing.sys.mjs", "RikkaEditing"],
  ["resource://gre/modules/RikkaCapture.sys.mjs", "RikkaCapture"],
  ["resource://gre/modules/RikkaCertificates.sys.mjs", "RikkaCertificates"],
]) {
  ChromeUtils.importESModule(module)[name].attach(window);
}
