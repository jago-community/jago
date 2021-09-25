const sending = browser.runtime.sendNativeMessage("jago", 'popup');

sending.then(
    function (got) { console.log(got) },
    function (error) { console.error(error) },
);

browser.tabs.query({
    currentWindow: true,
})
    .then(function (tabs) {
        console.log(tabs)
    })
    .catch(function (error) {
        console.error(error)
    });
