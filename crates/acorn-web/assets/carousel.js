(() => {
    const current = carousel => {
        const track = carousel.querySelector(".carousel-track");
        return track?.clientWidth ? Math.round(track.scrollLeft / track.clientWidth) : 0;
    };
    const update = carousel => {
        const slides = carousel.querySelectorAll(".carousel-slide");
        const index = Math.max(0, Math.min(current(carousel), slides.length - 1));
        const previous = carousel.querySelector('[data-carousel-action="previous"]');
        const next = carousel.querySelector('[data-carousel-action="next"]');
        const count = carousel.querySelector(".carousel-count");
        if (previous) previous.disabled = index === 0;
        if (next) next.disabled = index === slides.length - 1;
        if (count) count.textContent = `${index + 1} / ${slides.length}`;
    };
    document.addEventListener("click", event => {
        const button = event.target.closest?.("[data-carousel-action]");
        if (!button) return;
        const carousel = button.closest("[data-carousel]");
        const track = carousel?.querySelector(".carousel-track");
        if (!track) return;
        const direction = button.dataset.carouselAction === "next" ? 1 : -1;
        track.scrollTo({ left: (current(carousel) + direction) * track.clientWidth, behavior: "smooth" });
    });
    document.addEventListener("scroll", event => {
        if (event.target.classList?.contains("carousel-track")) update(event.target.closest("[data-carousel]"));
    }, true);
})();
