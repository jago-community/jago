import start, { dismantle, Context } from './web.js';

start()
    .then(() => {
        const host = browser.runtime.connectNative("jago");

        host.onMessage.addListener((message) => {
          console.info('native', message);
        });

        const context = Context.empty();

        browser.runtime.onConnect.addListener(port => {
            port.onMessage.addListener(([setting, message]) => {
                context.wrap(setting, message);

                host.postMessage([setting, message]);
            });
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
                        host.postMessage(part);
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
    })
    .catch(console.error)
