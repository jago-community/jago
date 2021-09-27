wasm_bindgen(browser.runtime.getURL("content/web_bg.wasm"))
    .then(() => {
        let port = browser.runtime.connect();

        port.onDisconnect.addListener((p) => {
          if (p.error) {
            console.log(`Disconnected due to an error: ${p.error.message}`);
          } else {
            console.log('port disconnected', p);
          }
        });

        port.onMessage.addListener((message) => {
          console.log('background', message);
        });

        wasm_bindgen.dismantle(window.document, (message) => {
            port.postMessage(message.toString());
        })
    })
    .catch(console.error)
