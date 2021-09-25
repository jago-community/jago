const script = document.createElement('script');
script.setAttribute("type", "module");
script.setAttribute("src", browser.extension.getURL('content/content.js'));

const body = document.body || document.getElementsByTagName("body")[0] || document.documentElement;
body.appendChild(script);
