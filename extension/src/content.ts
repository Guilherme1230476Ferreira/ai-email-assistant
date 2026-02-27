// Content script: scrapes email content and injects a "Generate Reply" button

function getEmailContent(): string {
  const emailBody = document.querySelector<HTMLElement>(".a3s.aiL");
  return emailBody?.innerText ?? "";
}

function addGenerateButton(): void {
  if (document.getElementById("ai-generate-btn")) return;

  const toolbar = document.querySelector<HTMLElement>(".btC");
  if (!toolbar) return;

  const button = document.createElement("button");
  button.id = "ai-generate-btn";
  button.textContent = "Generate Reply";
  button.style.cssText =
    "margin-left:8px;padding:6px 12px;background:#1a73e8;color:#fff;border:none;border-radius:4px;cursor:pointer;font-size:14px;";

  button.addEventListener("click", () => {
    const emailContent = getEmailContent();
    if (!emailContent) {
      alert("Could not find email content.");
      return;
    }

    button.textContent = "Generating…";
    button.disabled = true;

    chrome.runtime.sendMessage(
      { type: "GENERATE_REPLY", emailContent },
      (response: { reply?: string; error?: string }) => {
        button.textContent = "Generate Reply";
        button.disabled = false;

        if (response?.reply) {
          const replyBox = document.querySelector<HTMLElement>("[contenteditable='true']");
          if (replyBox) {
            replyBox.innerText = response.reply;
          } else {
            alert(response.reply);
          }
        } else {
          alert(`Error: ${response?.error ?? "Unknown error"}`);
        }
      }
    );
  });

  toolbar.appendChild(button);
}

// Observe DOM changes to handle Gmail's dynamic navigation
const observer = new MutationObserver(addGenerateButton);
observer.observe(document.body, { childList: true, subtree: true });
