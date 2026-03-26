document.addEventListener("DOMContentLoaded", function () {
  // Gallery carousel with auto-progress, swipe, and lightbox
  document.querySelectorAll("[data-gallery]").forEach(function (gallery) {
    var track = gallery.querySelector("[data-gallery-track]");
    var slides = track.querySelectorAll("[data-gallery-slide]");
    var prevBtn = gallery.querySelector("[data-gallery-prev]");
    var nextBtn = gallery.querySelector("[data-gallery-next]");
    var dotsContainer = gallery.querySelector("[data-gallery-dots]");
    var current = 0;
    var total = slides.length;
    var autoInterval = null;
    var touchStartX = 0;
    var touchEndX = 0;

    // Create dots
    for (var i = 0; i < total; i++) {
      var dot = document.createElement("button");
      dot.className = "gallery-dot";
      dot.setAttribute("aria-label", "Go to slide " + (i + 1));
      dot.dataset.index = i;
      dotsContainer.appendChild(dot);
    }
    var dots = dotsContainer.querySelectorAll(".gallery-dot");

    function goTo(index) {
      current = ((index % total) + total) % total;
      track.style.transform = "translateX(-" + (current * 100) + "%)";
      dots.forEach(function (d, i) {
        d.classList.toggle("active", i === current);
      });
    }

    function next() { goTo(current + 1); }
    function prev() { goTo(current - 1); }

    function startAuto() {
      stopAuto();
      autoInterval = setInterval(next, 4000);
    }

    function stopAuto() {
      if (autoInterval) clearInterval(autoInterval);
    }

    // Init
    goTo(0);
    startAuto();

    // Controls
    prevBtn.addEventListener("click", function () { prev(); startAuto(); });
    nextBtn.addEventListener("click", function () { next(); startAuto(); });
    dotsContainer.addEventListener("click", function (e) {
      if (e.target.dataset.index !== undefined) {
        goTo(parseInt(e.target.dataset.index));
        startAuto();
      }
    });

    // Swipe support
    track.addEventListener("touchstart", function (e) {
      touchStartX = e.changedTouches[0].screenX;
      stopAuto();
    }, { passive: true });
    track.addEventListener("touchend", function (e) {
      touchEndX = e.changedTouches[0].screenX;
      var diff = touchStartX - touchEndX;
      if (Math.abs(diff) > 50) {
        if (diff > 0) next(); else prev();
      }
      startAuto();
    }, { passive: true });

    // Pause on hover
    gallery.addEventListener("mouseenter", stopAuto);
    gallery.addEventListener("mouseleave", startAuto);

    // Lightbox on image click
    track.addEventListener("click", function (e) {
      var img = e.target.closest("img");
      if (!img) return;
      openLightbox(img.src, img.alt);
    });
  });

  // Lightbox
  var lightbox = document.createElement("div");
  lightbox.id = "gallery-lightbox";
  lightbox.innerHTML =
    '<div class="lightbox-backdrop"></div>' +
    '<div class="lightbox-content">' +
    '<button class="lightbox-close" aria-label="Close">&times;</button>' +
    '<img src="" alt="" />' +
    '</div>';
  document.body.appendChild(lightbox);

  var lbImg = lightbox.querySelector("img");
  var lbClose = lightbox.querySelector(".lightbox-close");
  var lbBackdrop = lightbox.querySelector(".lightbox-backdrop");

  function openLightbox(src, alt) {
    // Swap to largest available variant
    lbImg.src = src.replace(/_400w\.|_800w\./, "_1200w.");
    lbImg.alt = alt || "";
    lightbox.classList.add("active");
    document.body.style.overflow = "hidden";
  }

  function closeLightbox() {
    lightbox.classList.remove("active");
    document.body.style.overflow = "";
  }

  lbClose.addEventListener("click", closeLightbox);
  lbBackdrop.addEventListener("click", closeLightbox);
  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape") closeLightbox();
  });
});
