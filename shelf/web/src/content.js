
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

        wasm_bindgen.dismantle(window.document, (message) => {
            console.info(message);
            port.postMessage(message);
        })
    })
    .catch(console.error);
