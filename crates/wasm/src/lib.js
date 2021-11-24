import start from "./wasm.js";

function main() {
  if (!browser) {
    return;
  }

  const handle = browser.runtime.connectNative("jago");

  handle.onMessage.addListener((response) => {
    console.log("native: ", response);
  });

  handle.onDisconnect.addListener((v) => {
    console.log(v);
    console.log(browser.runtime.lastError);
  });

  browser.omnibox.onInputEntered.addListener((input) => {
    if (input === "g") {
      handle.postMessage("hello stranger");
    }
  });
}

start()
  .then(main)
  .catch((error) => console.error(error));
