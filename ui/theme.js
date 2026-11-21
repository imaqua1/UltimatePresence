// Applies the saved theme before first paint so the window never flashes the wrong one.
// Dark is the default; "system" follows Windows.
(function () {
  var t = "dark";
  try {
    var saved = localStorage.getItem("theme");
    if (saved === "light" || saved === "system" || saved === "dark") t = saved;
  } catch (e) {
    /* storage unavailable: keep dark */
  }
  document.documentElement.dataset.theme = t;
})();
