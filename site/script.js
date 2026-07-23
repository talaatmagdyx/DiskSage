const header = document.querySelector("[data-header]");
const menuButton = document.querySelector("[data-menu]");
const nav = document.querySelector("[data-nav]");

const updateHeader = () => header?.classList.toggle("scrolled", window.scrollY > 18);
updateHeader();
window.addEventListener("scroll", updateHeader, { passive: true });

menuButton?.addEventListener("click", () => {
  const open = menuButton.getAttribute("aria-expanded") !== "true";
  menuButton.setAttribute("aria-expanded", String(open));
  nav?.classList.toggle("open", open);
});

nav?.querySelectorAll("a").forEach((link) => link.addEventListener("click", () => {
  menuButton?.setAttribute("aria-expanded", "false");
  nav.classList.remove("open");
}));

const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
const reveals = document.querySelectorAll(".reveal");
if (reducedMotion || !("IntersectionObserver" in window)) {
  reveals.forEach((element) => element.classList.add("visible"));
} else {
  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (!entry.isIntersecting) return;
      entry.target.style.setProperty("--delay", `${entry.target.dataset.delay ?? 0}ms`);
      entry.target.classList.add("visible");
      observer.unobserve(entry.target);
    });
  }, { threshold: 0.12 });
  reveals.forEach((element) => observer.observe(element));
}

const tourImage = document.querySelector("[data-tour-image]");
const tourTitle = document.querySelector("[data-tour-title]");
const tourCopy = document.querySelector("[data-tour-copy]");
document.querySelectorAll("[data-shot]").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll("[data-shot]").forEach((candidate) => candidate.setAttribute("aria-selected", String(candidate === tab)));
    if (!tourImage || !tourTitle || !tourCopy) return;
    tourImage.style.opacity = "0";
    window.setTimeout(() => {
      tourImage.src = `assets/screenshots/${tab.dataset.shot}.png`;
      tourImage.alt = `DiskSage ${tab.textContent.trim()} screen`;
      tourTitle.textContent = tab.dataset.title;
      tourCopy.textContent = tab.dataset.copy;
      tourImage.style.opacity = "1";
    }, reducedMotion ? 0 : 140);
  });
});

document.querySelector("[data-year]").textContent = new Date().getFullYear();

fetch("https://api.github.com/repos/talaatmagdyx/DiskSage/releases/latest", { headers: { Accept: "application/vnd.github+json" } })
  .then((response) => response.ok ? response.json() : Promise.reject())
  .then((release) => {
    document.querySelectorAll("[data-download]").forEach((link) => { link.href = release.html_url; });
    const label = document.querySelector("[data-release-label]");
    if (label) label.textContent = `${release.tag_name} available`;
  })
  .catch(() => undefined);
