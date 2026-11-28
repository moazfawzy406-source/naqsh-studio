// Logo Grid Builder for Naqsh Vector. Clean-room port of the idea, written for the vc script API.
// Select your logo, then run: File > Scripts > logo-grid-builder.
// Builds a grid from the exact bounds of the selection on a NEW locked layer. The art is never edited.
// Options through args: mode (1-6), unit (pt|mm|px|in), pad, div, spokes, gfit (0-2), rfit (0-1), guides, lock.
// Modes: 1 Frame + measurements, 2 Modular, 3 Golden ratio, 4 Radial, 5 Isometric, 6 Clearspace.
(function () {
    var PHI = (1 + Math.sqrt(5)) / 2;
    var KAPPA = 0.5522847498;
    var COL_GRID = "#0096c8", COL_MAIN = "#e63c5a", COL_AUX = "#787878", COL_GOLD = "#eba000";
    var UNITS = { pt: 1, mm: 2.834645669, px: 1, "in": 72 };
    var NAMES = ["Frame", "Modular", "Golden", "Radial", "Isometric", "Clearspace"];
    var A = vc.args || {};

    function fail(msg) { throw new Error(msg); }
    function round2(v) { return Math.round(v * 100) / 100; }
    function rad(a) { return a * Math.PI / 180; }

    // ---- settings ----
    var mode = A.mode;
    if (mode === undefined || mode === null) {
        var ans = prompt("Grid type: 1 Frame, 2 Modular, 3 Golden, 4 Radial, 5 Isometric, 6 Clearspace", "1", "Logo Grid Builder");
        if (ans === null || ans === undefined) { return; }
        mode = parseInt(ans, 10);
    }
    mode = parseInt(mode, 10) - 1;
    if (isNaN(mode) || mode < 0 || mode > 5) { fail("Grid type must be a number from 1 to 6."); }
    var unit = A.unit && UNITS[A.unit] ? A.unit : "pt";
    var unitF = UNITS[unit];
    var s = {
        mode: mode, unit: unit, unitF: unitF,
        pad: (A.pad ? parseFloat(A.pad) : 0) * unitF,
        div: A.div ? parseInt(A.div, 10) : 8,
        spokes: A.spokes ? parseInt(A.spokes, 10) : 12,
        gfit: A.gfit ? parseInt(A.gfit, 10) : 0,
        rfit: A.rfit ? parseInt(A.rfit, 10) : 0,
        guides: !!A.guides,
        lock: A.lock === undefined ? true : !!A.lock
    };
    if (isNaN(s.div) || s.div < 1) { fail("Divisions must be a whole number, 1 or more."); }
    if (isNaN(s.pad) || s.pad < 0) { s.pad = 0; }
    if (isNaN(s.spokes) || s.spokes < 1) { s.spokes = 12; }
    function fmt(v) { return round2(v / s.unitF) + " " + s.unit; }

    // ---- bounds of the selection (document coordinates run downward; we work with y up) ----
    var info = vc.run("document.inspect", { depth: 0 });
    var sb = info && info.selectionBounds;
    if (!info || !info.selection || info.selection.length === 0 || !sb) {
        fail("Select your logo artwork first (all its parts), then run the script again.");
    }
    if (!(sb.width > 0) || !(sb.height > 0)) { fail("Could not measure the selection. Select artwork with a width and height."); }
    var B = { l: sb.x, r: sb.x + sb.width, t: -sb.y, b: -(sb.y + sb.height), w: sb.width, h: sb.height };
    B.cx = (B.l + B.r) / 2; B.cy = (B.t + B.b) / 2;
    var X = Math.max(B.w, B.h);
    var sw = Math.max(0.25, X / 800), fs = Math.max(6, X / 40), tick = Math.max(2, X / 100);

    // ---- drawing helpers: shapes are queued, then styled in batches and grouped ----
    var groups = [];
    function G(name) { var g = { name: name, ids: [], styled: {}, texts: [] }; groups.push(g); return g; }
    function note(g, id, color, wd, dash) {
        var k = color + "|" + wd + "|" + (dash ? dash.join(",") : "");
        if (!g.styled[k]) { g.styled[k] = { color: color, wd: wd, dash: dash, ids: [] }; }
        g.styled[k].ids.push(id);
        g.ids.push(id);
    }
    function line(g, x1, y1, x2, y2, color, wd, guide) {
        if (guide && Math.abs(x1 - x2) < 1e-9) { vc.run("guide.add", { vertical: true, pos: x1 }); return; }
        if (guide && Math.abs(y1 - y2) < 1e-9) { vc.run("guide.add", { vertical: false, pos: -y1 }); return; }
        var r = vc.run("shape.line", { x1: x1, y1: -y1, x2: x2, y2: -y2 });
        note(g, r.id, color, wd);
    }
    function circle(g, cx, cy, d, color, wd, guide) {
        var r = vc.run("shape.ellipse", { x: cx - d / 2, y: -cy - d / 2, width: d, height: d });
        note(g, r.id, color, wd);
    }
    function poly(g, pts, color, wd, dash) {
        var an = [], i;
        for (i = 0; i < pts.length; i++) { an.push({ x: pts[i][0], y: -pts[i][1] }); }
        var r = vc.run("path.create", { anchors: an, closed: true });
        note(g, r.id, color, wd, dash);
        return r.id;
    }
    function rectPath(g, l, b, r, t, color, wd, dash) { return poly(g, [[l, t], [r, t], [r, b], [l, b]], color, wd, dash); }
    function label(g, txt, x, y, size, color) {
        var r = vc.run("text.create", { x: x, y: -y, text: txt, size: size, color: color });
        g.texts.push(r.id);
        g.ids.push(r.id);
    }
    function dimH(g, x1, x2, y, txt, color) {
        line(g, x1, y, x2, y, color, sw, false);
        line(g, x1, y - tick, x1, y + tick, color, sw, false);
        line(g, x2, y - tick, x2, y + tick, color, sw, false);
        label(g, txt, (x1 + x2) / 2 - fs * txt.length * 0.25, y + tick + fs * 1.1, fs, color);
    }
    function dimV(g, y1, y2, x, txt, color) {
        line(g, x, y1, x, y2, color, sw, false);
        line(g, x - tick, y1, x + tick, y1, color, sw, false);
        line(g, x - tick, y2, x + tick, y2, color, sw, false);
        label(g, txt, x - tick - fs * txt.length * 0.55 - fs * 0.3, (y1 + y2) / 2 + fs * 0.4, fs, color);
    }
    function clipLine(nx, ny, c, hx, hy) {
        var pts = [], eps = 1e-6, i, v, xs = [-hx, hx], ys = [-hy, hy];
        if (Math.abs(ny) > 1e-9) { for (i = 0; i < 2; i++) { v = (c - nx * xs[i]) / ny; if (v >= -hy - eps && v <= hy + eps) { pts.push([xs[i], v]); } } }
        if (Math.abs(nx) > 1e-9) { for (i = 0; i < 2; i++) { v = (c - ny * ys[i]) / nx; if (v >= -hx - eps && v <= hx + eps) { pts.push([v, ys[i]]); } } }
        if (pts.length < 2) { return null; }
        var a = pts[0], best = null, bd = -1, d;
        for (i = 1; i < pts.length; i++) { d = Math.abs(pts[i][0] - a[0]) + Math.abs(pts[i][1] - a[1]); if (d > bd) { bd = d; best = pts[i]; } }
        if (bd < 1e-6) { return null; }
        return [a, best];
    }

    // ---- builders ----
    function buildFrame() {
        var g = G("Logo bounds + center + diagonals"), gG = G("Thirds + golden section lines"), gM = G("Measurements");
        var p = s.pad, i, v, gu = s.guides;
        rectPath(g, B.l, B.b, B.r, B.t, COL_MAIN, sw * 1.5);
        line(g, B.cx, B.b - p, B.cx, B.t + p, COL_MAIN, sw, gu);
        line(g, B.l - p, B.cy, B.r + p, B.cy, COL_MAIN, sw, gu);
        line(g, B.l, B.b, B.r, B.t, COL_AUX, sw, false);
        line(g, B.l, B.t, B.r, B.b, COL_AUX, sw, false);
        var fr = [1 / 3, 2 / 3], gr = [1 / PHI / PHI, 1 / PHI];
        for (i = 0; i < 2; i++) {
            v = fr[i];
            line(gG, B.l + v * B.w, B.b - p, B.l + v * B.w, B.t + p, COL_GRID, sw, gu);
            line(gG, B.l - p, B.b + v * B.h, B.r + p, B.b + v * B.h, COL_GRID, sw, gu);
            v = gr[i];
            line(gG, B.l + v * B.w, B.b - p, B.l + v * B.w, B.t + p, COL_GOLD, sw, gu);
            line(gG, B.l - p, B.b + v * B.h, B.r + p, B.b + v * B.h, COL_GOLD, sw, gu);
        }
        var off = Math.max(B.w, B.h) * 0.08 + p;
        dimH(gM, B.l, B.r, B.b - off, "W " + fmt(B.w), COL_MAIN);
        dimV(gM, B.b, B.t, B.l - off, "H " + fmt(B.h), COL_MAIN);
        label(gM, "W:H = 1 : " + round2(B.h / B.w) + "   (golden = 1 : 0.618)", B.l, B.t + off, fs, COL_AUX);
    }
    function buildModular() {
        var n = Math.min(s.div, 64), cell = X / n, gu = s.guides, i;
        var cols = Math.max(1, Math.ceil((B.w + 2 * s.pad) / cell - 1e-6));
        var rows = Math.max(1, Math.ceil((B.h + 2 * s.pad) / cell - 1e-6));
        var gw = cols * cell, gh = rows * cell, x0 = B.cx - gw / 2, y0 = B.cy - gh / 2;
        var g = G("Modular grid " + cols + "x" + rows + " cell " + fmt(cell));
        for (i = 0; i <= cols; i++) { line(g, x0 + i * cell, y0, x0 + i * cell, y0 + gh, COL_GRID, sw, gu); }
        for (i = 0; i <= rows; i++) { line(g, x0, y0 + i * cell, x0 + gw, y0 + i * cell, COL_GRID, sw, gu); }
        var gM = G("Logo bounds + measurements");
        rectPath(gM, B.l, B.b, B.r, B.t, COL_MAIN, sw * 1.5);
        line(gM, B.cx, y0, B.cx, y0 + gh, COL_MAIN, sw, gu);
        line(gM, x0, B.cy, x0 + gw, B.cy, COL_MAIN, sw, gu);
        var off = cell * 0.6;
        dimH(gM, x0, x0 + cell, y0 - off, "cell " + fmt(cell), COL_MAIN);
        dimH(gM, B.l, B.r, y0 - off * 2.6, "W " + fmt(B.w), COL_MAIN);
        dimV(gM, B.b, B.t, x0 - off, "H " + fmt(B.h), COL_MAIN);
    }
    function buildGolden() {
        var W, h;
        if (s.gfit === 0) { W = B.w; } else if (s.gfit === 1) { W = B.h * PHI; } else { W = Math.max(B.w, B.h); }
        h = W / PHI;
        var n = Math.max(2, Math.min(s.div, 14)), L = B.cx - W / 2, Bt = B.cy - h / 2;
        var gRect = G("Golden squares"), gCirc = G("Golden inscribed circles"), gSpir = G("Golden spiral"), gAux = G("Golden rectangle + logo bounds");
        var r = h, a = 180, ccx = L + h, ccy = Bt + h;
        var anchors = [], rights = [], lefts = [], k, ax, ay, bx, by;
        for (k = 0; k < n; k++) {
            ax = Math.cos(rad(a)); ay = Math.sin(rad(a)); bx = Math.cos(rad(a + 90)); by = Math.sin(rad(a + 90));
            var p0 = [ccx + r * ax, ccy + r * ay], p3 = [ccx + r * bx, ccy + r * by];
            poly(gRect, [[ccx, ccy], [ccx + r * ax, ccy + r * ay], [ccx + r * (ax + bx), ccy + r * (ay + by)], [ccx + r * bx, ccy + r * by]], COL_GRID, sw);
            circle(gCirc, ccx + r * (ax + bx) / 2, ccy + r * (ay + by) / 2, r, COL_AUX, sw, false);
            if (k === 0) { anchors.push(p0); }
            anchors.push(p3);
            rights.push([p0[0] - r * KAPPA * ay, p0[1] + r * KAPPA * ax]);
            lefts.push([p3[0] + r * KAPPA * by, p3[1] - r * KAPPA * bx]);
            var r2 = r / PHI, a2 = a + 90;
            ccx = p3[0] - r2 * Math.cos(rad(a2)); ccy = p3[1] - r2 * Math.sin(rad(a2));
            r = r2; a = a2;
        }
        var an = [];
        for (k = 0; k < anchors.length; k++) {
            var pt = { x: anchors[k][0], y: -anchors[k][1] };
            if (k < n) { pt.out = [rights[k][0], -rights[k][1]]; }
            if (k > 0) { pt["in"] = [lefts[k - 1][0], -lefts[k - 1][1]]; }
            an.push(pt);
        }
        var sp = vc.run("path.create", { anchors: an, closed: false });
        note(gSpir, sp.id, COL_MAIN, sw * 2);
        rectPath(gAux, L, Bt, L + W, Bt + h, COL_GOLD, sw * 1.5);
        line(gAux, L, Bt, L + W, Bt + h, COL_AUX, sw, false);
        line(gAux, L, Bt + h, L + W, Bt, COL_AUX, sw, false);
        rectPath(gAux, B.l, B.b, B.r, B.t, COL_MAIN, sw);
        var off = Math.max(W, B.h) * 0.06;
        dimH(gAux, L, L + W, Bt - off, "golden W " + fmt(W) + "  /  H " + fmt(h), COL_GOLD);
    }
    function buildRadial() {
        var R, n = Math.min(s.div, 40), spk = Math.min(s.spokes, 180), k, i, gu = s.guides;
        R = s.rfit === 0 ? Math.sqrt(B.w * B.w + B.h * B.h) / 2 : Math.max(B.w, B.h) / 2;
        R += s.pad;
        var gR = G("Radial rings " + n + " step " + fmt(R / n)), gS = G("Radial spokes " + spk), gM = G("Logo bounds + measurements");
        for (i = 1; i <= n; i++) { circle(gR, B.cx, B.cy, 2 * R * i / n, COL_GRID, sw, gu); }
        for (k = 0; k < spk; k++) {
            var a = rad(k * 360 / spk);
            line(gS, B.cx, B.cy, B.cx + R * Math.cos(a), B.cy + R * Math.sin(a), COL_AUX, sw, false);
        }
        rectPath(gM, B.l, B.b, B.r, B.t, COL_MAIN, sw * 1.5);
        line(gM, B.l, B.b, B.r, B.t, COL_MAIN, sw, false);
        line(gM, B.l, B.t, B.r, B.b, COL_MAIN, sw, false);
        dimH(gM, B.cx, B.cx + R, B.cy - R - R * 0.08, "R " + fmt(R) + "  (step " + fmt(R / n) + ")", COL_MAIN);
    }
    function buildIso() {
        var n = Math.min(s.div, 60), m, f, seg, c, cell = X / n, p = cell * Math.sqrt(3) / 2;
        var hx = B.w / 2 + s.pad, hy = B.h / 2 + s.pad, M = Math.ceil((hx + hy) / p) + 1;
        var norms = [[-1, 0], [-0.5, Math.sqrt(3) / 2], [-0.5, -Math.sqrt(3) / 2]];
        var names = ["Isometric vertical", "Isometric 30 deg", "Isometric 150 deg"];
        for (f = 0; f < 3; f++) {
            var g = G(names[f]);
            for (m = -M; m <= M; m++) {
                c = m * p;
                seg = clipLine(norms[f][0], norms[f][1], c, hx, hy);
                if (seg) { line(g, B.cx + seg[0][0], B.cy + seg[0][1], B.cx + seg[1][0], B.cy + seg[1][1], COL_GRID, sw, false); }
            }
        }
        var gM = G("Logo bounds + measurements");
        rectPath(gM, B.l, B.b, B.r, B.t, COL_MAIN, sw * 1.5);
        rectPath(gM, B.cx - hx, B.cy - hy, B.cx + hx, B.cy + hy, COL_AUX, sw);
        dimH(gM, B.l, B.r, B.b - hy * 0.2 - s.pad, "W " + fmt(B.w) + "   triangle side " + fmt(cell), COL_MAIN);
    }
    function buildClearspace() {
        var i, g = G("Clearspace X = " + fmt(X)), ratios = [1, 0.5, 0.25], cols = [COL_MAIN, COL_GRID, COL_AUX];
        rectPath(g, B.l, B.b, B.r, B.t, COL_MAIN, sw * 1.5);
        for (i = 0; i < 3; i++) {
            var d = X * ratios[i];
            rectPath(g, B.l - d, B.b - d, B.r + d, B.t + d, cols[i], sw * 1.5, [sw * 8, sw * 5]);
        }
        var gM = G("Clearspace measurements");
        dimH(gM, B.r, B.r + X / 4, B.t + X / 8, "X/4 " + fmt(X / 4), COL_AUX);
        dimH(gM, B.r + X / 4, B.r + X / 2, B.t + X / 8, "X/2", COL_GRID);
        dimH(gM, B.r + X / 2, B.r + X, B.t + X / 8, "X " + fmt(X), COL_MAIN);
        dimH(gM, B.l, B.r, B.b - X - fs * 3, "W " + fmt(B.w), COL_MAIN);
        dimV(gM, B.b, B.t, B.l - X - fs * 3, "H " + fmt(B.h), COL_MAIN);
        label(gM, "X = larger side of logo = " + fmt(X), B.l - X, B.t + X + fs * 2, fs, COL_MAIN);
    }

    // ---- run: new layer, build, style, group, lock ----
    var layer = vc.run("layer.new", { name: "LOGO GRID - " + NAMES[mode] });
    var layerId = layer && layer.id;
    switch (mode) {
        case 0: buildFrame(); break;
        case 1: buildModular(); break;
        case 2: buildGolden(); break;
        case 3: buildRadial(); break;
        case 4: buildIso(); break;
        default: buildClearspace(); break;
    }
    var total = 0, gi, k2;
    for (gi = 0; gi < groups.length; gi++) {
        var g = groups[gi];
        if (g.ids.length === 0) { continue; }
        for (k2 in g.styled) {
            var st = g.styled[k2];
            vc.run("select.set", { ids: st.ids });
            vc.run("paint.setFill", { none: true });
            vc.run("paint.setStroke", { color: st.color });
            vc.run("stroke.set", st.dash ? { weight: st.wd, dash: st.dash } : { weight: st.wd });
        }
        total += g.ids.length;
        vc.run("select.set", { ids: g.ids });
        var gr = vc.run("object.group", {});
        if (gr && gr.id !== undefined) { vc.run("object.setProps", { id: gr.id, name: g.name }); }
    }
    vc.run("select.none", {});
    if (s.lock && layerId !== undefined) { vc.run("object.setProps", { id: layerId, locked: true }); }
    vc.log("Logo Grid Builder: " + NAMES[mode] + " grid, " + total + " objects, logo " + fmt(B.w) + " x " + fmt(B.h));
})();
