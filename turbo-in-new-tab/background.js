chrome.tabs.onCreated.addListener(function (tab) {
 if (tab.pendingUrl === "chrome://newtab/") {
  chrome.tabs.create({ url: "http://localhost:3000/websearch" });
  chrome.tabs.remove(tab.id);
 }
});
