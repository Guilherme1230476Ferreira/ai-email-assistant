// Service worker for the AI Email Assistant Chrome extension (Manifest V3)

chrome.runtime.onInstalled.addListener(() => {
  console.log("AI Email Assistant installed.");
});

chrome.runtime.onMessage.addListener(
  (
    message: { type: string; emailContent?: string },
    _sender,
    sendResponse: (response: { reply?: string; error?: string }) => void
  ) => {
    if (message.type === "GENERATE_REPLY" && message.emailContent) {
      // Forward request to backend API
      fetch("http://localhost:3000/api/generate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ content: message.emailContent }),
      })
        .then((res) => res.json())
        .then((data: { reply: string }) => sendResponse({ reply: data.reply }))
        .catch((err: Error) => sendResponse({ error: err.message }));

      // Return true to indicate async response
      return true;
    }
  }
);
