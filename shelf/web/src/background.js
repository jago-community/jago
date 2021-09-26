import { consume, dismantle } from './web.js';


browser.runtime.onMessage.addListener((message) => {
  console.info('content', message);
});

const port = browser.runtime.connectNative("jago");

port.onMessage.addListener((message) => {
  console.info('native', message);
});

function handle(input) {
    input = input.trim();

    if (input === 'open') {
        browser.browserAction.openPopup();
    } else if (input.startsWith('o') || input.startsWith('open')) {
        const word_break = input.indexOf(' ');
        const rest = input.slice(word_break) ;
        browser.tabs.create({
            active: true,
            url: browser.runtime.getURL(rest),
        });
    } else if (input === 'i') {
        let views = browser
            .extension
            .getViews();

        for (let i = 0; i < views.length; i++) {
            const view = views[i];

            dismantle(view.document, (part) => {
                port.postMessage(part);
            });
        }
    }
}

browser.omnibox.onInputEntered.addListener(handle);

browser.commands.onCommand.addListener((input) => {
    if (input === 'debug-popup') {
        input = 'open popup/mod.html';
    }

    handle(input);
});
