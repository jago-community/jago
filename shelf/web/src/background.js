import { consume, dismantle } from './web.js';

browser.commands.onCommand.addListener(function (command) {
  if (command === "open") {
    browser.browserAction.openPopup();
  }
});

const port = browser.runtime.connectNative("jago");

port.onMessage.addListener((message) => {
  console.info('native', message);
});

browser.omnibox.onInputEntered.addListener(function (input) {
    if (input.trim() === 'i') {
        let views = browser
            .extension
            .getViews();
        for (let i = 0; i < views.length; i++) {
            const view = views[i];
            dismantle(view.document, (part) => {
                console.log('handled', part);
            });
        }
    }
});
