const board = document.getElementById("board");
const toolbar = document.getElementById("toolbar");
const editToggle = document.getElementById("editToggle");
const clearBtn = document.getElementById("clearBtn");
const toolButtons = Array.from(document.querySelectorAll(".tool-btn[data-tool]"));
const drawSizeInput = document.getElementById("drawSize");
const drawColorInput = document.getElementById("drawColor");
const textFontInput = document.getElementById("textFont");
const textSizeInput = document.getElementById("textSize");
const textColorInput = document.getElementById("textColor");
const drawControls = document.getElementById("drawControls");
const textControls = document.getElementById("textControls");

let isEditing = false;
let currentTool = "move";

let drawing = null;
let drawingLayer = null;
const history = [];
let historyIndex = -1;
let restoringHistory = false;
let maxZIndex = 0;
let activeTextNode = null;

function bringItemToFront(item) {
    maxZIndex += 1;
    item.style.zIndex = String(maxZIndex);
}

function getFontFamilyFromValue(value) {
    if (value === "serif") {
        return '"Iowan Old Style", "Times New Roman", serif';
    }

    if (value === "mono") {
        return '"IBM Plex Mono", "Fira Code", monospace';
    }

    if (value === "comic") {
        return '"Comic Sans MS", "Chalkboard SE", cursive';
    }

    return '"IBM Plex Sans", "Segoe UI", sans-serif';
}

function styleTextNode(node) {
    node.style.color = textColorInput.value;
    node.style.fontSize = `${textSizeInput.value}px`;
    node.style.fontFamily = getFontFamilyFromValue(textFontInput.value);
}

function syncTextControlsFromNode(node) {
    const color = node.style.color;
    const size = parseFloat(node.style.fontSize || "18");
    const family = node.style.fontFamily || "";

    if (color) {
        const probe = document.createElement("div");
        probe.style.color = color;
        document.body.appendChild(probe);
        const computed = getComputedStyle(probe).color;
        document.body.removeChild(probe);
        const rgb = computed.match(/\d+/g);
        if (rgb && rgb.length >= 3) {
            textColorInput.value = `#${rgb
                .slice(0, 3)
                .map((v) => Number(v).toString(16).padStart(2, "0"))
                .join("")}`;
        }
    }

    textSizeInput.value = String(clamp(size, 12, 52));

    if (family.includes("Mono") || family.includes("monospace")) {
        textFontInput.value = "mono";
    } else if (family.includes("Times") || family.includes("serif")) {
        textFontInput.value = "serif";
    } else if (family.includes("Comic") || family.includes("cursive")) {
        textFontInput.value = "comic";
    } else {
        textFontInput.value = "plex";
    }
}

function applyTextControlsToActive() {
    if (!activeTextNode || !isEditing) {
        return;
    }

    styleTextNode(activeTextNode);
    const item = activeTextNode.closest(".item");
    if (item) {
        applyItemLayout(item);
        updateItemRatios(item);
    }
}

function updateStyleControlsVisibility() {
    const showDraw = isEditing && currentTool === "draw";
    const showText = isEditing && currentTool === "text";

    drawControls.classList.toggle("hidden", !showDraw);
    textControls.classList.toggle("hidden", !showText);
}

function setEditing(on) {
    isEditing = on;
    editToggle.setAttribute("aria-pressed", String(on));
    editToggle.textContent = on ? "Done" : "Edit";
    toolbar.classList.toggle("hidden", !on);

    document.querySelectorAll(".item").forEach((item) => {
        item.classList.toggle("editing", on);
        const deleteBtn = item.querySelector(".item-delete");
        const resizeHandle = item.querySelector(".item-resize");
        if (deleteBtn) {
            deleteBtn.style.display = on ? "grid" : "none";
        }
        if (resizeHandle) {
            resizeHandle.style.display = on ? "block" : "none";
        }
    });

    if (!on) {
        stopDrawingPreview();
    }

    updateStyleControlsVisibility();
}

function setTool(tool) {
    currentTool = tool;
    toolButtons.forEach((btn) => {
        btn.classList.toggle("active", btn.dataset.tool === tool);
    });
    updateStyleControlsVisibility();
}

function pointInBoard(evt) {
    const rect = board.getBoundingClientRect();
    return {
        x: evt.clientX - rect.left,
        y: evt.clientY - rect.top,
    };
}

function clamp(val, min, max) {
    return Math.max(min, Math.min(max, val));
}

function updateItemRatios(item) {
    const boardWidth = Math.max(board.clientWidth, 1);
    const boardHeight = Math.max(board.clientHeight, 1);
    const left = parseFloat(item.style.left || "0");
    const top = parseFloat(item.style.top || "0");

    item.dataset.xRatio = String(left / boardWidth);
    item.dataset.yRatio = String(top / boardHeight);

    if (item.dataset.kind === "draw" || item.dataset.kind === "image" || item.dataset.kind === "video") {
        const width = item.getBoundingClientRect().width;
        item.dataset.widthRatio = String(width / boardWidth);
    } else if (item.dataset.kind === "text" && item.dataset.hasCustomWidth === "1") {
        const width = item.getBoundingClientRect().width;
        item.dataset.widthRatio = String(width / boardWidth);
    }
}

function applyItemLayout(item) {
    const boardWidth = Math.max(board.clientWidth, 1);
    const boardHeight = Math.max(board.clientHeight, 1);
    const xRatio = parseFloat(item.dataset.xRatio || "0");
    const yRatio = parseFloat(item.dataset.yRatio || "0");
    if (item.dataset.kind === "draw" || item.dataset.kind === "image" || item.dataset.kind === "video") {
        const intrinsicWidth = parseFloat(item.dataset.intrinsicWidth || "0");
        const intrinsicHeight = parseFloat(item.dataset.intrinsicHeight || "0");
        const widthRatio = parseFloat(item.dataset.widthRatio || "0");
        const finalWidth = clamp(boardWidth * widthRatio, 96, boardWidth * 0.98);

        item.style.width = `${finalWidth}px`;
        if (intrinsicWidth > 0 && intrinsicHeight > 0) {
            item.style.height = `${(finalWidth * intrinsicHeight) / intrinsicWidth}px`;
        }
    } else if (item.dataset.kind === "text") {
        if (item.dataset.hasCustomWidth === "1") {
            const widthRatio = parseFloat(item.dataset.widthRatio || "0");
            const width = clamp(boardWidth * widthRatio, 80, boardWidth * 0.92);
            item.style.width = `${width}px`;
        } else {
            item.style.width = "";
        }
    }

    const itemRect = item.getBoundingClientRect();
    const nextLeft = clamp(xRatio * boardWidth, 0, Math.max(0, boardWidth - itemRect.width));
    const nextTop = clamp(yRatio * boardHeight, 0, Math.max(0, boardHeight - itemRect.height));

    item.style.left = `${nextLeft}px`;
    item.style.top = `${nextTop}px`;
}

function applyResponsiveLayout() {
    document.querySelectorAll(".item").forEach((item) => {
        applyItemLayout(item);
    });
}

function serializeBoard() {
    document.querySelectorAll(".item").forEach((item) => updateItemRatios(item));
    return Array.from(document.querySelectorAll(".item")).map((item) => {
        const state = {
            kind: item.dataset.kind || "text",
            xRatio: parseFloat(item.dataset.xRatio || "0"),
            yRatio: parseFloat(item.dataset.yRatio || "0"),
            widthRatio: parseFloat(item.dataset.widthRatio || "0"),
            intrinsicWidth: parseFloat(item.dataset.intrinsicWidth || "0"),
            intrinsicHeight: parseFloat(item.dataset.intrinsicHeight || "0"),
            zIndex: parseInt(item.style.zIndex || "0", 10),
        };

        if (state.kind === "text") {
            const textNode = item.querySelector(".text-content");
            state.text = textNode ? textNode.textContent : "";
            state.textColor = textNode ? textNode.style.color || "" : "";
            state.textSize = textNode ? textNode.style.fontSize || "" : "";
            state.textFont = textNode ? textNode.style.fontFamily || "" : "";
            state.hasCustomWidth = item.dataset.hasCustomWidth || "0";
        } else {
            const media = item.querySelector("img");
            state.src = media ? media.src : item.dataset.src || "";
            state.stroke = parseFloat(item.dataset.stroke || "2");
            state.strokeColor = item.dataset.strokeColor || "#1f1f1f";
        }

        return state;
    });
}

function pushHistory() {
    if (restoringHistory) {
        return;
    }

    const snapshot = serializeBoard();
    const encoded = JSON.stringify(snapshot);
    const current = historyIndex >= 0 ? JSON.stringify(history[historyIndex]) : "";
    if (encoded === current) {
        return;
    }

    history.splice(historyIndex + 1);
    history.push(snapshot);
    historyIndex = history.length - 1;
}

function restoreHistory(snapshot) {
    restoringHistory = true;
    document.querySelectorAll(".item").forEach((node) => node.remove());
    snapshot.forEach((state) => createItemFromState(state));
    restoringHistory = false;
}

function undoHistory() {
    if (historyIndex <= 0) {
        return;
    }

    historyIndex -= 1;
    restoreHistory(history[historyIndex]);
}

function makeItem(x, y, contentNode, kind, explicitZIndex) {
    const item = document.createElement("div");
    item.className = "item";
    item.style.left = `${x}px`;
    item.style.top = `${y}px`;
    item.dataset.kind = kind;
    if (typeof explicitZIndex === "number") {
        item.style.zIndex = String(explicitZIndex);
        maxZIndex = Math.max(maxZIndex, explicitZIndex);
    } else {
        bringItemToFront(item);
    }

    const resizeHandle = document.createElement("div");
    resizeHandle.className = "item-resize";
    resizeHandle.style.display = isEditing ? "block" : "none";

    const content = document.createElement("div");
    content.className = "item-content";
    content.appendChild(contentNode);

    const deleteBtn = document.createElement("button");
    deleteBtn.className = "item-delete";
    deleteBtn.type = "button";
    deleteBtn.textContent = "x";
    deleteBtn.setAttribute("aria-label", "Delete element");
    deleteBtn.style.display = isEditing ? "grid" : "none";
    deleteBtn.addEventListener("click", (evt) => {
        evt.stopPropagation();
        if (!isEditing) {
            return;
        }
        item.remove();
        pushHistory();
    });

    item.appendChild(deleteBtn);
    item.appendChild(resizeHandle);
    item.appendChild(content);
    board.appendChild(item);

    setupDrag(item);
    setupResize(item, resizeHandle);
    item.classList.toggle("editing", isEditing);
    updateItemRatios(item);
    applyItemLayout(item);

    return item;
}

function setupResize(item, resizeHandle) {
    let resize = null;
    let changed = false;

    resizeHandle.addEventListener("pointerdown", (evt) => {
        if (!isEditing) {
            return;
        }

        evt.preventDefault();
        evt.stopPropagation();
        bringItemToFront(item);

        const rect = item.getBoundingClientRect();
        const boardRect = board.getBoundingClientRect();
        resize = {
            startX: evt.clientX,
            startWidth: rect.width,
            boardWidth: boardRect.width,
        };
        changed = false;
        resizeHandle.setPointerCapture(evt.pointerId);
    });

    resizeHandle.addEventListener("pointermove", (evt) => {
        if (!resize) {
            return;
        }

        const deltaX = evt.clientX - resize.startX;
        const nextWidth = clamp(resize.startWidth + deltaX, 80, resize.boardWidth * 0.98);

        item.dataset.widthRatio = String(nextWidth / Math.max(resize.boardWidth, 1));
        if (item.dataset.kind === "text") {
            item.dataset.hasCustomWidth = "1";
        }

        applyItemLayout(item);
        updateItemRatios(item);
        changed = true;
    });

    const stop = () => {
        if (resize && changed) {
            pushHistory();
        }
        resize = null;
        changed = false;
    };

    resizeHandle.addEventListener("pointerup", stop);
    resizeHandle.addEventListener("pointercancel", stop);
}

function setupDrag(item) {
    let drag = null;
    let moved = false;

    item.addEventListener("pointerdown", (evt) => {
        if (!isEditing || currentTool !== "move") {
            return;
        }

        if (evt.target.closest(".item-delete") || evt.target.closest(".item-resize")) {
            return;
        }

        if (evt.target.closest(".text-content")) {
            return;
        }

        evt.preventDefault();
        bringItemToFront(item);
        const rect = item.getBoundingClientRect();
        const boardRect = board.getBoundingClientRect();

        drag = {
            offsetX: evt.clientX - rect.left,
            offsetY: evt.clientY - rect.top,
            width: rect.width,
            height: rect.height,
            boardRect,
        };
        moved = false;

        item.setPointerCapture(evt.pointerId);
    });

    item.addEventListener("pointermove", (evt) => {
        if (!drag) {
            return;
        }

        const x = evt.clientX - drag.boardRect.left - drag.offsetX;
        const y = evt.clientY - drag.boardRect.top - drag.offsetY;

        item.style.left = `${clamp(x, 0, drag.boardRect.width - drag.width)}px`;
        item.style.top = `${clamp(y, 0, drag.boardRect.height - drag.height)}px`;
        updateItemRatios(item);
        moved = true;
    });

    const stop = () => {
        if (drag && moved) {
            pushHistory();
        }
        drag = null;
        moved = false;
    };

    item.addEventListener("pointerup", stop);
    item.addEventListener("pointercancel", stop);
}

function createTextAt(x, y) {
    const text = document.createElement("div");
    text.className = "text-content";
    text.contentEditable = "true";
    text.spellcheck = false;
    text.textContent = "Text";
    styleTextNode(text);

    const item = makeItem(x, y, text, "text");

    text.addEventListener("input", () => {
        applyItemLayout(item);
        updateItemRatios(item);
    });

    text.addEventListener("focus", () => {
        activeTextNode = text;
        syncTextControlsFromNode(text);
    });

    text.addEventListener("blur", () => {
        applyItemLayout(item);
        updateItemRatios(item);
        pushHistory();
    });

    text.focus();
    const selection = window.getSelection();
    const range = document.createRange();
    range.selectNodeContents(text);
    selection.removeAllRanges();
    selection.addRange(range);

    pushHistory();

    return item;
}

function startDrawingPreview() {
    if (drawingLayer) {
        return;
    }

    const canvas = document.createElement("canvas");
    canvas.className = "drawing-layer";
    canvas.width = board.clientWidth;
    canvas.height = board.clientHeight;
    board.appendChild(canvas);
    drawingLayer = canvas;
}

function stopDrawingPreview() {
    if (drawingLayer) {
        drawingLayer.remove();
        drawingLayer = null;
    }
    drawing = null;
}

function redrawPreview() {
    if (!drawing || !drawingLayer) {
        return;
    }

    const ctx = drawingLayer.getContext("2d");
    ctx.clearRect(0, 0, drawingLayer.width, drawingLayer.height);

    if (drawing.points.length < 2) {
        return;
    }

    ctx.lineWidth = drawing.size;
    ctx.strokeStyle = drawing.color;
    ctx.lineJoin = "round";
    ctx.lineCap = "round";
    ctx.beginPath();

    const first = drawing.points[0];
    ctx.moveTo(first.x, first.y);

    for (let i = 1; i < drawing.points.length; i += 1) {
        const p = drawing.points[i];
        ctx.lineTo(p.x, p.y);
    }

    ctx.stroke();
}

function drawingToSvg(points, color, size) {
    const pad = Math.max(8, size * 2);
    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    points.forEach((p) => {
        minX = Math.min(minX, p.x);
        minY = Math.min(minY, p.y);
        maxX = Math.max(maxX, p.x);
        maxY = Math.max(maxY, p.y);
    });

    const w = Math.max(24, maxX - minX + pad * 2);
    const h = Math.max(24, maxY - minY + pad * 2);

    const d = points
        .map((p, i) => {
            const x = (p.x - minX + pad).toFixed(1);
            const y = (p.y - minY + pad).toFixed(1);
            return i === 0 ? `M ${x} ${y}` : `L ${x} ${y}`;
        })
        .join(" ");

    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}"><path d="${d}" fill="none" stroke="${color}" stroke-width="${size}" stroke-linecap="round" stroke-linejoin="round"/></svg>`;

    return {
        svg,
        x: minX - pad,
        y: minY - pad,
        width: w,
        height: h,
    };
}

function extractYouTubeId(url) {
    try {
        const parsed = new URL(url);
        const host = parsed.hostname.replace(/^www\./, "");

        if (host === "youtu.be") {
            const id = parsed.pathname.split("/").filter(Boolean)[0];
            return id || null;
        }

        if (host === "youtube.com" || host === "m.youtube.com") {
            if (parsed.pathname === "/watch") {
                return parsed.searchParams.get("v");
            }

            if (parsed.pathname.startsWith("/embed/")) {
                return parsed.pathname.split("/")[2] || null;
            }

            if (parsed.pathname.startsWith("/shorts/")) {
                return parsed.pathname.split("/")[2] || null;
            }
        }
    } catch (_err) {
        return null;
    }

    return null;
}

function createVideoAt(x, y, url) {
    const videoId = extractYouTubeId(url);
    if (!videoId) {
        window.alert("Please provide a valid YouTube URL.");
        return;
    }

    const wrapper = document.createElement("div");
    wrapper.className = "video-content";

    const frame = document.createElement("div");
    frame.className = "video-frame";

    const iframe = document.createElement("iframe");
    iframe.src = `https://www.youtube.com/embed/${videoId}`;
    iframe.allow = "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share";
    iframe.allowFullscreen = true;
    iframe.title = "YouTube video player";

    frame.appendChild(iframe);
    wrapper.appendChild(frame);

    const item = makeItem(x, y, wrapper, "video");
    item.dataset.widthRatio = String(0.55);
    item.dataset.intrinsicWidth = String(560);
    item.dataset.intrinsicHeight = String(315);
    item.dataset.src = iframe.src;

    applyItemLayout(item);
    updateItemRatios(item);
    pushHistory();
}

function placeDrawing(points) {
    if (points.length < 2) {
        return;
    }

    const out = drawingToSvg(points, drawing.color, drawing.size);
    const img = document.createElement("img");
    img.className = "draw-content";
    img.alt = "drawing";
    img.src = `data:image/svg+xml;base64,${btoa(out.svg)}`;
    const item = makeItem(
        clamp(out.x, 0, board.clientWidth - out.width),
        clamp(out.y, 0, board.clientHeight - out.height),
        img,
        "draw",
    );

    item.dataset.intrinsicWidth = String(out.width);
    item.dataset.intrinsicHeight = String(out.height);
    item.dataset.widthRatio = String(Math.min(out.width / Math.max(board.clientWidth, 1), 0.92));
    item.dataset.stroke = String(drawing.size);
    item.dataset.strokeColor = drawing.color;
    applyItemLayout(item);
    pushHistory();
}

function createImageAt(x, y, src) {
    const img = document.createElement("img");
    img.className = "image-content";
    img.alt = "image";
    img.src = src;

    const item = makeItem(x, y, img, "image");
    item.dataset.widthRatio = String(0.45);

    img.addEventListener("load", () => {
        item.dataset.intrinsicWidth = String(img.naturalWidth || 240);
        item.dataset.intrinsicHeight = String(img.naturalHeight || 160);
        applyItemLayout(item);
        updateItemRatios(item);
        pushHistory();
    });

    img.addEventListener("error", () => {
        item.remove();
    });

    pushHistory();
}

function createItemFromState(state) {
    const boardWidth = Math.max(board.clientWidth, 1);
    const boardHeight = Math.max(board.clientHeight, 1);
    const x = (state.xRatio || 0) * boardWidth;
    const y = (state.yRatio || 0) * boardHeight;

    if (state.kind === "text") {
        const text = document.createElement("div");
        text.className = "text-content";
        text.contentEditable = "true";
        text.spellcheck = false;
        text.textContent = state.text || "";
        if (state.textColor) {
            text.style.color = state.textColor;
        }
        if (state.textSize) {
            text.style.fontSize = state.textSize;
        }
        if (state.textFont) {
            text.style.fontFamily = state.textFont;
        }

        const item = makeItem(x, y, text, "text", state.zIndex);
        item.dataset.xRatio = String(state.xRatio || 0);
        item.dataset.yRatio = String(state.yRatio || 0);
        item.dataset.widthRatio = String(state.widthRatio || 0);
        item.dataset.hasCustomWidth = state.hasCustomWidth || "0";
        applyItemLayout(item);

        text.addEventListener("input", () => {
            applyItemLayout(item);
            updateItemRatios(item);
        });

        text.addEventListener("focus", () => {
            activeTextNode = text;
            syncTextControlsFromNode(text);
        });

        text.addEventListener("blur", () => {
            applyItemLayout(item);
            updateItemRatios(item);
            pushHistory();
        });
        return;
    }

    let mediaNode;
    if (state.kind === "video") {
        const wrapper = document.createElement("div");
        wrapper.className = "video-content";
        const frame = document.createElement("div");
        frame.className = "video-frame";
        const iframe = document.createElement("iframe");
        iframe.src = state.src || "";
        iframe.allow = "accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share";
        iframe.allowFullscreen = true;
        iframe.title = "YouTube video player";
        frame.appendChild(iframe);
        wrapper.appendChild(frame);
        mediaNode = wrapper;
    } else {
        const img = document.createElement("img");
        img.className = state.kind === "draw" ? "draw-content" : "image-content";
        img.alt = state.kind;
        img.src = state.src || "";
        mediaNode = img;
    }

    const item = makeItem(x, y, mediaNode, state.kind, state.zIndex);
    item.dataset.xRatio = String(state.xRatio || 0);
    item.dataset.yRatio = String(state.yRatio || 0);
    item.dataset.widthRatio = String(state.widthRatio || 0.45);
    item.dataset.intrinsicWidth = String(state.intrinsicWidth || 240);
    item.dataset.intrinsicHeight = String(state.intrinsicHeight || 160);
    item.dataset.stroke = String(state.stroke || 2);
    item.dataset.strokeColor = state.strokeColor || "#1f1f1f";
    item.dataset.src = state.src || "";
    applyItemLayout(item);
}

board.addEventListener("pointerdown", (evt) => {
    if (!isEditing || evt.target.closest(".item")) {
        return;
    }

    const p = pointInBoard(evt);

    if (currentTool === "text") {
        createTextAt(p.x, p.y);
        return;
    }

    if (currentTool === "draw") {
        startDrawingPreview();
        drawing = {
            points: [p],
            color: drawColorInput.value,
            size: parseFloat(drawSizeInput.value || "2"),
        };
        board.setPointerCapture(evt.pointerId);
        return;
    }

    if (currentTool === "image") {
        const url = window.prompt("Image/GIF URL");
        if (!url) {
            return;
        }
        createImageAt(p.x, p.y, url.trim());
        return;
    }

    if (currentTool === "video") {
        const url = window.prompt("YouTube URL");
        if (!url) {
            return;
        }
        createVideoAt(p.x, p.y, url.trim());
    }
});

board.addEventListener("pointerdown", (evt) => {
    const textNode = evt.target.closest(".text-content");
    if (!textNode) {
        return;
    }

    activeTextNode = textNode;
    syncTextControlsFromNode(textNode);
});

board.addEventListener("pointermove", (evt) => {
    if (!drawing) {
        return;
    }

    drawing.points.push(pointInBoard(evt));
    redrawPreview();
});

function finalizeDrawing() {
    if (!drawing) {
        return;
    }
    placeDrawing(drawing.points);
    stopDrawingPreview();
}

board.addEventListener("pointerup", finalizeDrawing);
board.addEventListener("pointercancel", finalizeDrawing);

window.addEventListener("resize", () => {
    if (drawingLayer) {
        drawingLayer.width = board.clientWidth;
        drawingLayer.height = board.clientHeight;
        redrawPreview();
    }

    applyResponsiveLayout();
});

editToggle.addEventListener("click", () => {
    setEditing(!isEditing);
    if (isEditing && currentTool !== "move") {
        setTool("move");
    }
});

toolButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
        if (!isEditing) {
            return;
        }
        setTool(btn.dataset.tool);
    });
});

clearBtn.addEventListener("click", () => {
    if (!isEditing) {
        return;
    }
    document.querySelectorAll(".item").forEach((node) => node.remove());
    stopDrawingPreview();
    pushHistory();
});

textFontInput.addEventListener("change", () => {
    applyTextControlsToActive();
    pushHistory();
});

textSizeInput.addEventListener("input", () => {
    applyTextControlsToActive();
});

textSizeInput.addEventListener("change", () => {
    applyTextControlsToActive();
    pushHistory();
});

textColorInput.addEventListener("input", () => {
    applyTextControlsToActive();
});

textColorInput.addEventListener("change", () => {
    applyTextControlsToActive();
    pushHistory();
});

window.addEventListener("keydown", (evt) => {
    const isUndo = (evt.ctrlKey || evt.metaKey) && !evt.shiftKey && evt.key.toLowerCase() === "z";
    if (!isUndo) {
        return;
    }

    evt.preventDefault();
    undoHistory();
});

setEditing(false);
setTool("move");
pushHistory();
