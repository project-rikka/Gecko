/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

"use strict";

document.documentElement.setAttribute("windowclass", `rikka-view-${window.name}`);

ChromeUtils.importESModule("resource://gre/modules/RikkaUserContent.sys.mjs").RikkaUserContent.attach(window);
ChromeUtils.importESModule("resource://gre/modules/RikkaView.sys.mjs").RikkaView.attach(window);
