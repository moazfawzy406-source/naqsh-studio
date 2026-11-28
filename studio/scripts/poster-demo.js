// Naqsh Vector real-use check: builds a simple Arabic poster from shapes and text.
(function () {
    function fillOnly(id, color) {
        vc.run("select.set", { ids: [id] });
        vc.run("paint.setFill", { color: color });
        vc.run("paint.setStroke", { none: true });
    }
    vc.run("file.new", { name: "poster", width: 595, height: 842 });
    var bg = vc.run("shape.rectangle", { x: 0, y: 0, width: 595, height: 842 });
    fillOnly(bg.id, "#1b1f3b");
    var sun = vc.run("shape.ellipse", { x: 117, y: 150, width: 360, height: 360 });
    fillOnly(sun.id, "#f5b82e");
    var star = vc.run("shape.star", { cx: 297, cy: 330, radius1: 140, radius2: 60, points: 8 });
    fillOnly(star.id, "#e63c5a");
    var dot = vc.run("shape.ellipse", { x: 257, y: 290, width: 80, height: 80 });
    fillOnly(dot.id, "#ffffff");
    var bar = vc.run("shape.rectangle", { x: 60, y: 640, width: 475, height: 6, radius: 3 });
    fillOnly(bar.id, "#f5b82e");
    var t1 = vc.run("text.create", { x: 535, y: 600, text: "\u0646\u0642\u0634 \u0633\u062a\u0648\u062f\u064a\u0648", size: 72, color: "#ffffff" });
    var t2 = vc.run("text.create", { x: 535, y: 700, text: "\u0643\u0644 \u0627\u0644\u062a\u0635\u0645\u064a\u0645 \u0641\u064a \u0628\u0631\u0646\u0627\u0645\u062c \u0648\u0627\u062d\u062f", size: 28, color: "#f5b82e" });
    var t3 = vc.run("text.create", { x: 60, y: 790, text: "Naqsh Studio  -  one window, four apps", size: 18, color: "#aab0d0" });
    vc.run("select.none", {});
    vc.log("poster built: " + [bg.id, sun.id, star.id, dot.id, bar.id, t1.id, t2.id, t3.id].join(","));
})();
