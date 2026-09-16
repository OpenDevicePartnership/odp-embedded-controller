(function () {
    var script = document.createElement("script");
    script.src = "https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js";
    script.onload = function () {
        mermaid.initialize({ startOnLoad: false, theme: "default" });
        document.querySelectorAll("pre code.language-mermaid").forEach(function (code) {
            var pre = code.parentElement;
            var graphDiv = document.createElement("div");
            graphDiv.className = "mermaid";
            graphDiv.textContent = code.textContent;
            pre.replaceWith(graphDiv);
        });
        mermaid.run({ querySelector: ".mermaid" });
    };
    document.head.appendChild(script);
})();
