// v 私の好み
document.body.removeChild(document.body.firstChild);
const buttonsDiv = document.createElement("div");
{
  const addClickHandlerAndAppendDiv = (element, handler) => {
    element.addEventListener("click", handler);
    buttonsDiv.appendChild(element);
  };
  const createRoomButton = document.createElement("span");
  createRoomButton.textContent = "部屋を作成";
  addClickHandlerAndAppendDiv(createRoomButton, () => {
    location.href = "/room/new";
  });
  const howToPlayButton = document.createElement("span");
  howToPlayButton.textContent = "遊び方(未実装)";
  addClickHandlerAndAppendDiv(howToPlayButton, () => {});
}
document.body.appendChild(buttonsDiv);
