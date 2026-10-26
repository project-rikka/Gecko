/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

export class RikkaCaptureProcessChild extends JSProcessActorChild {
  observe(subject, topic) {
    if (topic !== "recording-device-events") {
      return;
    }
    let window;
    try {
      window = subject.QueryInterface(Ci.nsIPropertyBag2).getProperty("window");
    } catch {
      return;
    }
    const top = window?.browsingContext?.top?.window;
    const reported = top && !Cu.isRemoteProxy(top) ? top : window;
    try {
      reported?.windowGlobalChild?.getActor("RikkaCapture").report();
    } catch {}
  }
}
