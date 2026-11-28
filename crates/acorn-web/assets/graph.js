// Force layout and pointer controls for the RAD graph. The SVG and its accessible
// entity list remain owned by Dioxus; this module only moves diagram geometry.
(() => {
    let active = null;
    let scanQueued = false;
    let remembered = null;
    const storageKey = 'acorn.graph.view.v1';
    const bounded = value => Number.isFinite(value) && Math.abs(value) <= 100000;
    const rememberView = view => {
        if (!view) return remembered;
        const positions = new Map(remembered?.signature === view.signature ? remembered.positions : []);
        for (const [id, position] of view.positions) positions.set(id, position);
        remembered = { signature: view.signature, camera: { ...view.camera }, positions };
        return remembered;
    };
    const readView = (signature, key, nodes) => {
        try {
            const raw = localStorage.getItem(storageKey);
            if (!raw || raw.length > 500000) return null;
            const saved = JSON.parse(raw);
            const camera = saved?.camera;
            const positions = saved?.nodes;
            if (!bounded(camera?.x) || !bounded(camera?.y) || !Number.isFinite(camera?.size) || camera.size < 160 || camera.size > 1600 || !Array.isArray(positions)) return null;
            if (saved.version === 1 && (saved.key !== key || positions.length !== nodes.length || !positions.every((position, index) => position.id === nodes[index].id))) return null;
            if (saved.version === 2 && saved.signature !== signature) return null;
            if (![1, 2].includes(saved.version) || !positions.every(position => typeof position.id === 'string' && bounded(position.x) && bounded(position.y))) return null;
            return { signature, camera, positions: new Map(positions.map(({ id, x, y }) => [id, { x, y }])) };
        } catch { return null; }
    };
    window.acornGraphView = { clear() { active?.stopSaving(); remembered = null; try { localStorage.removeItem(storageKey); return true; } catch { return false; } } };

    const scan = () => {
        scanQueued = false;
        const svg = document.querySelector('.graph-svg');
        const key = svg ? [
            ...svg.querySelectorAll('.graph-node'),
            ...svg.querySelectorAll('.graph-edge'),
        ].map(element => element.dataset.nodeId || element.dataset.edgeId).join('|') : '';
        const signature = svg?.dataset.graphSignature ?? '';
        if (active?.svg === svg && active.key === key && active.signature === signature) {
            active.updateSelection();
            return;
        }
        rememberView(active?.snapshot());
        active?.dispose();
        active = svg && key ? createGraph(svg, key, signature) : null;
    };
    const queueScan = () => {
        if (!scanQueued) {
            scanQueued = true;
            queueMicrotask(scan);
        }
    };

    function createGraph(svg, key, signature) {
        const nodes = [...svg.querySelectorAll('.graph-node')].map((element, index) => {
            const x = Number(element.querySelector('circle')?.getAttribute('cx')) || 400 + index;
            const y = Number(element.querySelector('circle')?.getAttribute('cy')) || 400 + index;
            return { element, id: element.dataset.nodeId, x, y, initialX: Number(element.dataset.originX), initialY: Number(element.dataset.originY), vx: 0, vy: 0 };
        });
        const restored = remembered?.signature === signature ? remembered : readView(signature, key, nodes);
        if (restored) nodes.forEach(node => {
            const position = restored.positions.get(node.id);
            if (position) { node.x = position.x; node.y = position.y; }
        });
        const byId = new Map(nodes.map(node => [node.id, node]));
        const edges = [...svg.querySelectorAll('.graph-edge')].map(element => ({
            element,
            id: element.dataset.edgeId,
            source: byId.get(element.dataset.source),
            target: byId.get(element.dataset.target),
        })).filter(edge => edge.source && edge.target);
        const edgeById = new Map(edges.map(edge => [edge.id, edge]));
        const panel = svg.closest('.graph-layout').querySelector('.graph-side-panel');
        let hoveredId = null;
        let hoveredEdgeId = null;
        const updateSelection = () => {
            if (hoveredEdgeId && !panel.querySelector('.graph-relationships')) hoveredEdgeId = null;
            const hoveredEdge = edgeById.get(hoveredEdgeId);
            const activeId = hoveredId || (svg.dataset.focused === 'true' ? null : svg.querySelector('.graph-node.selected')?.dataset.nodeId);
            const connected = new Set(hoveredEdge ? [hoveredEdge.source.id, hoveredEdge.target.id] : activeId ? [activeId] : []);
            if (activeId && !hoveredEdge) {
                const radius = Math.max(1, Number.parseInt(svg.dataset.focusRadius, 10) || 1);
                let frontier = new Set([activeId]);
                for (let hop = 0; hop < radius && frontier.size; hop += 1) {
                    const next = new Set();
                    for (const { source, target } of edges) {
                        if (frontier.has(source.id) || frontier.has(target.id)) {
                            if (!connected.has(source.id)) next.add(source.id);
                            if (!connected.has(target.id)) next.add(target.id);
                        }
                    }
                    for (const id of next) connected.add(id);
                    frontier = next;
                }
            }
            for (const edge of edges) {
                const { element, source, target } = edge;
                const emphasized = hoveredEdge ? edge === hoveredEdge : activeId && connected.has(source.id) && connected.has(target.id);
                element.style.opacity = hoveredEdge || activeId ? emphasized ? '1' : '0.1' : '';
                element.toggleAttribute('data-hovered', edge === hoveredEdge);
            }
            for (const node of nodes) {
                node.element.style.opacity = (hoveredEdge || activeId) && !connected.has(node.id) ? '0.32' : '';
                node.element.toggleAttribute('data-hovered', !hoveredEdge && node.id === hoveredId);
                node.element.toggleAttribute('data-relationship-hovered', Boolean(hoveredEdge) && connected.has(node.id));
            }
        };
        const entityItem = target => target instanceof Element ? target.closest('.graph-item[data-graph-node-id]') : null;
        const relationshipItem = target => target instanceof Element ? target.closest('.graph-relationships [data-graph-edge-id]') : null;
        const setHover = (nodeId, edgeId) => {
            const nextNode = byId.has(nodeId) ? nodeId : null;
            const nextEdge = edgeById.has(edgeId) ? edgeId : null;
            if (hoveredId !== nextNode || hoveredEdgeId !== nextEdge) {
                hoveredId = nextNode;
                hoveredEdgeId = nextEdge;
                updateSelection();
            }
        };
        const onPanelEnter = event => setHover(entityItem(event.target)?.dataset.graphNodeId, relationshipItem(event.target)?.dataset.graphEdgeId);
        const onPanelLeave = event => setHover(entityItem(event.relatedTarget)?.dataset.graphNodeId, relationshipItem(event.relatedTarget)?.dataset.graphEdgeId);
        const onEntityClick = event => { if (entityItem(event.target)) setHover(null, null); };
        const svgNode = target => target instanceof Element ? target.closest('.graph-node[data-node-id]') : null;
        const onNodeEnter = event => setHover(svgNode(event.target)?.dataset.nodeId, null);
        const onNodeLeave = event => setHover(svgNode(event.relatedTarget)?.dataset.nodeId, null);
        let frame = 0;
        let alpha = restored ? 0 : 1;
        let pointer = null;
        const touches = new Map();
        let gesture = null;
        let suppressClick = false;
        let initialFit = !restored;
        let camera = restored ? { ...restored.camera } : { x: 0, y: 0, size: 800 };
        let saveTimer = 0;
        let saving = true;
        const snapshot = () => saving ? {
            signature,
            camera,
            positions: new Map(nodes.map(({ id, x, y }) => [id, { x, y }])),
        } : null;
        const save = () => {
            clearTimeout(saveTimer);
            saveTimer = 0;
            if (saving) {
                const view = rememberView(snapshot());
                try { localStorage.setItem(storageKey, JSON.stringify({ version: 2, signature, camera: view.camera, nodes: [...view.positions].map(([id, position]) => ({ id, ...position })) })); } catch {}
            }
        };
        const scheduleSave = () => { clearTimeout(saveTimer); saveTimer = setTimeout(save, 200); };
        const mini = svg.closest('.graph-visual').querySelector('.graph-minimap-svg');
        const miniCircle = nodes.map(node => {
            const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            circle.setAttribute('class', 'graph-mini-node');
            circle.setAttribute('r', '3');
            circle.style.fill = getComputedStyle(node.element.querySelector('circle')).fill;
            mini.append(circle);
            return circle;
        });
        const miniViewport = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        miniViewport.setAttribute('class', 'graph-mini-viewport');
        mini.append(miniViewport);
        const renderMinimap = () => {
            const xs = nodes.map(node => node.x);
            const ys = nodes.map(node => node.y);
            const left = Math.min(...xs) - 45;
            const top = Math.min(...ys) - 45;
            const width = Math.max(180, Math.max(...xs) - Math.min(...xs) + 90);
            const height = Math.max(120, Math.max(...ys) - Math.min(...ys) + 90);
            const scale = Math.min(128 / width, 80 / height);
            const offsetX = 8 + (128 - width * scale) / 2;
            const offsetY = 8 + (80 - height * scale) / 2;
            for (let index = 0; index < nodes.length; index += 1) {
                miniCircle[index].setAttribute('cx', offsetX + (nodes[index].x - left) * scale);
                miniCircle[index].setAttribute('cy', offsetY + (nodes[index].y - top) * scale);
            }
            miniViewport.setAttribute('x', offsetX + (camera.x - left) * scale);
            miniViewport.setAttribute('y', offsetY + (camera.y - top) * scale);
            miniViewport.setAttribute('width', camera.size * scale);
            miniViewport.setAttribute('height', camera.size * scale);
        };
        const renderCamera = () => {
            svg.setAttribute('viewBox', `${camera.x} ${camera.y} ${camera.size} ${camera.size}`);
            renderMinimap();
        };
        const svgPoint = event => {
            const point = svg.createSVGPoint();
            point.x = event.clientX;
            point.y = event.clientY;
            return point.matrixTransform(svg.getScreenCTM().inverse());
        };
        const draw = () => {
            for (const { element, source, target } of edges) {
                const dx = target.x - source.x;
                const dy = target.y - source.y;
                const length = Math.max(1, Math.hypot(dx, dy));
                element.setAttribute('x1', source.x + dx * 17 / length);
                element.setAttribute('y1', source.y + dy * 17 / length);
                element.setAttribute('x2', target.x - dx * 21 / length);
                element.setAttribute('y2', target.y - dy * 21 / length);
            }
            for (const node of nodes) {
                node.element.querySelector('circle')?.setAttribute('cx', node.x);
                node.element.querySelector('circle')?.setAttribute('cy', node.y);
                const mark = node.element.querySelector('.graph-node-mark');
                mark?.setAttribute('transform', `translate(${node.x - 8.5} ${node.y - 8.5}) scale(0.7)`);
                const label = node.element.querySelector('.graph-node-label');
                label?.setAttribute('x', node.x);
                label?.setAttribute('y', node.y + 30);
            }
            renderMinimap();
        };
        const tick = () => {
            frame = 0;
            if (alpha < 0.015 && !pointer?.node) {
                svg.dataset.layoutSettled = 'true';
                if (initialFit) fit();
                initialFit = false;
                save();
                return;
            }
            for (let i = 0; i < nodes.length; i += 1) {
                for (let j = i + 1; j < nodes.length; j += 1) {
                    const a = nodes[i];
                    const b = nodes[j];
                    const dx = b.x - a.x || (j - i) * 0.01;
                    const dy = b.y - a.y || 0.01;
                    const distance2 = Math.max(100, dx * dx + dy * dy);
                    const force = alpha * 2800 / (distance2 * Math.sqrt(distance2));
                    a.vx -= dx * force;
                    a.vy -= dy * force;
                    b.vx += dx * force;
                    b.vy += dy * force;
                }
            }
            for (const { source, target } of edges) {
                const dx = target.x - source.x;
                const dy = target.y - source.y;
                const distance = Math.max(1, Math.hypot(dx, dy));
                const force = alpha * 0.012 * (distance - 125) / distance;
                source.vx += dx * force;
                source.vy += dy * force;
                target.vx -= dx * force;
                target.vy -= dy * force;
            }
            for (const node of nodes) {
                node.vx = (node.vx + (400 - node.x) * alpha * 0.002) * 0.82;
                node.vy = (node.vy + (400 - node.y) * alpha * 0.002) * 0.82;
                if (pointer?.node !== node) {
                    node.x += node.vx;
                    node.y += node.vy;
                }
            }
            alpha *= 0.976;
            draw();
            frame = requestAnimationFrame(tick);
        };
        const reheat = () => {
            alpha = Math.max(alpha, 0.32);
            svg.dataset.layoutSettled = 'false';
            if (!frame) frame = requestAnimationFrame(tick);
        };
        const fit = () => {
            const xs = nodes.map(node => node.x);
            const ys = nodes.map(node => node.y);
            const x0 = Math.min(...xs) - 70;
            const x1 = Math.max(...xs) + 70;
            const y0 = Math.min(...ys) - 70;
            const y1 = Math.max(...ys) + 70;
            const size = Math.max(180, x1 - x0, y1 - y0);
            camera = { x: (x0 + x1 - size) / 2, y: (y0 + y1 - size) / 2, size };
            renderCamera();
        };
        const reset = () => {
            clearTimeout(saveTimer);
            try { localStorage.removeItem(storageKey); } catch {}
            cancelAnimationFrame(frame);
            frame = 0;
            alpha = 1;
            pointer = null;
            initialFit = true;
            for (const node of nodes) {
                node.x = node.initialX;
                node.y = node.initialY;
                node.vx = 0;
                node.vy = 0;
            }
            svg.dataset.layoutSettled = 'false';
            draw();
            fit();
            frame = requestAnimationFrame(tick);
        };
        const zoom = (factor, event) => {
            const old = event ? svgPoint(event) : { x: camera.x + camera.size / 2, y: camera.y + camera.size / 2 };
            const size = Math.max(160, Math.min(1600, camera.size * factor));
            const ratio = size / camera.size;
            camera = { x: old.x - (old.x - camera.x) * ratio, y: old.y - (old.y - camera.y) * ratio, size };
            renderCamera();
        };
        const onWheel = event => {
            event.preventDefault();
            initialFit = false;
            const delta = event.deltaMode === 1 ? event.deltaY * 16 : event.deltaMode === 2 ? event.deltaY * svg.clientHeight : event.deltaY;
            zoom(Math.exp(Math.max(-240, Math.min(240, delta)) * 0.00045), event);
            scheduleSave();
        };
        const touchPair = () => {
            const [first, second] = [...touches.values()];
            return {
                x: (first.clientX + second.clientX) / 2,
                y: (first.clientY + second.clientY) / 2,
                distance: Math.max(1, Math.hypot(first.clientX - second.clientX, first.clientY - second.clientY)),
            };
        };
        const onPointerDown = event => {
            if (event.button !== 0) return;
            initialFit = false;
            const element = event.target.closest?.('.graph-node');
            const node = element && byId.get(element.dataset.nodeId);
            (element || svg).setPointerCapture(event.pointerId);
            if (event.pointerType === 'touch') touches.set(event.pointerId, { clientX: event.clientX, clientY: event.clientY });
            if (touches.size >= 2) {
                pointer = null;
                gesture = touchPair();
                suppressClick = true;
                setHover(null, null);
            } else {
                suppressClick = false;
                const point = svgPoint(event);
                pointer = { id: event.pointerId, node, x: point.x, y: point.y, clientX: event.clientX, clientY: event.clientY, startX: event.clientX, startY: event.clientY, moved: false };
                if (node) {
                    node.vx = 0;
                    node.vy = 0;
                    reheat();
                }
            }
        };
        const onPointerMove = event => {
            if (event.pointerType === 'touch' && touches.has(event.pointerId)) {
                touches.set(event.pointerId, { clientX: event.clientX, clientY: event.clientY });
            }
            if (event.pointerType === 'touch' && touches.size >= 2 && gesture) {
                const next = touchPair();
                const before = svgPoint({ clientX: gesture.x, clientY: gesture.y });
                const after = svgPoint({ clientX: next.x, clientY: next.y });
                const size = Math.max(160, Math.min(1600, camera.size * gesture.distance / next.distance));
                const ratio = size / camera.size;
                camera = { x: before.x - (after.x - camera.x) * ratio, y: before.y - (after.y - camera.y) * ratio, size };
                gesture = next;
                suppressClick = true;
                renderCamera();
                scheduleSave();
            } else if (pointer?.id === event.pointerId) {
                pointer.moved ||= Math.hypot(event.clientX - pointer.startX, event.clientY - pointer.startY) > 4;
                const point = svgPoint(event);
                const dx = point.x - pointer.x;
                const dy = point.y - pointer.y;
                if (pointer.node) {
                    pointer.node.x += dx;
                    pointer.node.y += dy;
                    draw();
                } else {
                    const pixelsPerUnit = Math.min(svg.clientWidth, svg.clientHeight) / camera.size;
                    camera.x -= (event.clientX - pointer.clientX) / pixelsPerUnit;
                    camera.y -= (event.clientY - pointer.clientY) / pixelsPerUnit;
                    renderCamera();
                }
                pointer.x = point.x;
                pointer.y = point.y;
                pointer.clientX = event.clientX;
                pointer.clientY = event.clientY;
            }
        };
        const onPointerUp = event => {
            if (event.pointerType === 'touch') touches.delete(event.pointerId);
            if (gesture && event.pointerType === 'touch') {
                gesture = touches.size >= 2 ? touchPair() : null;
                if (touches.size === 1) {
                    const [id, touch] = touches.entries().next().value;
                    const point = svgPoint(touch);
                    pointer = { id, node: null, x: point.x, y: point.y, clientX: touch.clientX, clientY: touch.clientY, startX: touch.clientX, startY: touch.clientY, moved: true };
                }
                suppressClick = true;
                scheduleSave();
            } else if (pointer?.id === event.pointerId) {
                const wasNode = pointer.node;
                suppressClick = pointer.moved;
                pointer = null;
                if (wasNode) reheat();
                scheduleSave();
            }
        };
        const onClickCapture = event => {
            if (suppressClick) {
                event.preventDefault();
                event.stopPropagation();
            }
        };
        const controls = svg.closest('.graph-visual');
        const fullscreenButton = controls.querySelector('[data-graph-action="fullscreen"]');
        const onFullscreenChange = () => {
            const fullscreen = document.fullscreenElement === controls;
            fullscreenButton.setAttribute('aria-label', fullscreen ? 'Exit fullscreen graph' : 'Fullscreen graph');
            fullscreenButton.title = fullscreen ? 'Exit fullscreen graph' : 'Fullscreen graph';
        };
        const onControl = event => {
            const action = event.target.closest?.('[data-graph-action]')?.dataset.graphAction;
            if (action && action !== 'reset' && action !== 'fullscreen') initialFit = false;
            if (action === 'zoom-in') zoom(0.8);
            if (action === 'zoom-out') zoom(1.25);
            if (action === 'fit') fit();
            if (action === 'reset') reset();
            if (action === 'fullscreen') {
                if (document.fullscreenElement === controls) document.exitFullscreen().catch(() => {});
                else controls.requestFullscreen?.().catch(() => {});
            }
            if (['zoom-in', 'zoom-out', 'fit'].includes(action)) scheduleSave();
        };
        svg.addEventListener('wheel', onWheel, { passive: false });
        svg.addEventListener('pointerdown', onPointerDown);
        svg.addEventListener('pointermove', onPointerMove);
        svg.addEventListener('pointerup', onPointerUp);
        svg.addEventListener('pointercancel', onPointerUp);
        svg.addEventListener('click', onClickCapture, true);
        svg.addEventListener('mouseover', onNodeEnter);
        svg.addEventListener('mouseout', onNodeLeave);
        controls.addEventListener('click', onControl);
        document.addEventListener('fullscreenchange', onFullscreenChange);
        panel.addEventListener('mouseover', onPanelEnter);
        panel.addEventListener('mouseout', onPanelLeave);
        panel.addEventListener('focusin', onPanelEnter);
        panel.addEventListener('focusout', onPanelLeave);
        panel.addEventListener('click', onEntityClick);
        renderCamera();
        draw();
        updateSelection();
        if (restored) svg.dataset.layoutSettled = 'true';
        else frame = requestAnimationFrame(tick);
        return {
            svg,
            key,
            signature,
            snapshot,
            updateSelection,
            stopSaving() { saving = false; clearTimeout(saveTimer); saveTimer = 0; },
            dispose() {
                if (saveTimer) save();
                cancelAnimationFrame(frame);
                svg.removeEventListener('wheel', onWheel);
                svg.removeEventListener('pointerdown', onPointerDown);
                svg.removeEventListener('pointermove', onPointerMove);
                svg.removeEventListener('pointerup', onPointerUp);
                svg.removeEventListener('pointercancel', onPointerUp);
                svg.removeEventListener('click', onClickCapture, true);
                svg.removeEventListener('mouseover', onNodeEnter);
                svg.removeEventListener('mouseout', onNodeLeave);
                controls.removeEventListener('click', onControl);
                document.removeEventListener('fullscreenchange', onFullscreenChange);
                panel.removeEventListener('mouseover', onPanelEnter);
                panel.removeEventListener('mouseout', onPanelLeave);
                panel.removeEventListener('focusin', onPanelEnter);
                panel.removeEventListener('focusout', onPanelLeave);
                panel.removeEventListener('click', onEntityClick);
                mini.replaceChildren();
            },
        };
    }

    new MutationObserver(queueScan).observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: ['class', 'data-focused', 'data-focus-radius'] });
    queueScan();
})();
