/**
 * Manages multiple employee selection in the work form.
 */
(() => {
  document.addEventListener("DOMContentLoaded", () => {
    const form = document.querySelector('form[action="/works"]');
    const select = document.querySelector("#performer-select");
    const container = document.querySelector("#performer-chips");
    const hidden = document.querySelector("#employee-ids");

    if (!form || !select || !container || !hidden) {
      return;
    }

    const selected = new Map();
    const maxPerformers = 20;

    /**
     * Renders selected employees and updates submitted IDs.
     */
    function render() {
      container.replaceChildren();

      for (const [id, name] of selected) {
        const chip = document.createElement("span");
        chip.className = "performer-chip";

        const label = document.createElement("span");
        label.textContent = name;

        const remove = document.createElement("button");
        remove.type = "button";
        remove.textContent = "×";
        remove.setAttribute("aria-label", `Удалить исполнителя ${name}`);

        remove.addEventListener("click", () => {
          selected.delete(id);
          render();
        });

        chip.append(label, remove);
        container.append(chip);
      }

      hidden.value = [...selected.keys()].join(",");
    }

    select.addEventListener("change", () => {
      const id = select.value;
      const option = select.selectedOptions[0];

      if (!id || !option) {
        return;
      }

      if (selected.size >= maxPerformers && !selected.has(id)) {
        alert("Можно выбрать не более 20 исполнителей");
        select.value = "";
        return;
      }

      selected.set(id, option.textContent.trim());

      select.value = "";
      select.setCustomValidity("");

      render();
    });

    form.addEventListener("submit", (event) => {
      if (selected.size === 0) {
        event.preventDefault();

        select.setCustomValidity("Выберите хотя бы одного исполнителя");
        select.reportValidity();
      }
    });
  });
})();
