import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import vm from 'node:vm';
import { Simulator } from '../docs/simulation-fallback.js';

/** Load browser helpers in a minimal host without starting I/O or animation. */
async function browserHarness({ reducedMotion = false } = {}) {
    const makeElement = () => ({
        children: [],
        textContent: '',
        append(...children) { this.children.push(...children); },
        replaceChildren(...children) { this.children = children; },
    });
    const handlers = new Map();
    const elements = new Map();
    const element = selector => {
        if (!elements.has(selector)) {
            elements.set(selector, {
                ...makeElement(),
                addEventListener: (event, handler) => handlers.set(`${selector}:${event}`, handler),
                classList: { add() {}, remove() {}, toggle() {} },
                setAttribute() {},
                getBoundingClientRect: () => ({ left: 20, top: 30 }),
                setPointerCapture() {}, releasePointerCapture() {},
                querySelector: () => null,
            });
        }

        return elements.get(selector);
    };
    const source = (await readFile(new URL('../docs/app.js', import.meta.url), 'utf8'))
        .replace(/^import .*;\n/m, '')
        .replace(/boot\(\)\.catch\([\s\S]*$/, '');
    const context = vm.createContext({
        window: {
            matchMedia: query => ({
                matches: query === '(prefers-reduced-motion: reduce)' && reducedMotion,
                addEventListener: (event, handler) => handlers.set(`${query}:${event}`, handler),
            }),
            addEventListener: (event, handler) => handlers.set(`window:${event}`, handler),
        },
        document: {
            querySelector: element, querySelectorAll: () => [],
            createElement: makeElement, body: element('body'),
        },
        performance: { now: () => 0 }, URL,
    });
    vm.runInContext(source, context);
    vm.runInContext('bindEvents()', context);

    return { context, handlers, elements, evaluate: code => vm.runInContext(code, context) };
}

test('fallback motion remains deterministic, bounded, and resettable', () => {
    const anchors = [0, 0, 2200, 1900];
    const left = new Simulator(anchors, anchors, [0, 1], [1], 42, 2200, 1900);
    const right = new Simulator(anchors, anchors, [0, 1], [1], 42, 2200, 1900);

    for (let iteration = 0; iteration < 1000; iteration++) {
        const actual = left.tick(.05);
        const expected = right.tick(.05);

        assert.deepEqual(actual, expected);
        assert.ok(actual.every((value, index) => value >= 0 && value <= (index % 2 ? 1900 : 2200)));
    }

    assert.deepEqual([...left.reset()], anchors);
});

test('fallback rejects malformed inputs and ignores nonfinite time', () => {
    const anchors = [100, 100];
    const simulator = new Simulator(anchors, anchors, [], []);

    const unchanged = simulator.tick(Number.NaN);

    assert.deepEqual([...unchanged], anchors);
    assert.throws(() => new Simulator([NaN, 0], anchors, [], []), RangeError);
    assert.throws(() => new Simulator(anchors, anchors, [0, 1], [1]), RangeError);
    assert.throws(() => new Simulator(anchors, anchors, [], [], 1, -1, 100), RangeError);
    assert.throws(() => new Simulator(anchors, [0], [], []), RangeError);
});

test('a tap updates its coordinates without any pointermove', async () => {
    const harness = await browserHarness();
    harness.evaluate(`app.layer = 'systems'; app.size = {width:1800,height:1680,fit:1};
        app.nodes = []; app.positions = [];`);
    const event = { clientX: 220, clientY: 330, pointerId: 1, target: { closest: () => null } };

    harness.handlers.get('#stage:pointerdown')(event);
    const x = harness.evaluate('app.pointer.x');
    const y = harness.evaluate('app.pointer.y');
    harness.handlers.get('#stage:pointerup')(event);

    assert.equal(x, 200);
    assert.equal(y, 300);
    assert.equal(harness.evaluate('app.pointer.down'), false);
});

test('canceled gestures and panel controls cannot select a node', async () => {
    const harness = await browserHarness();
    harness.evaluate("app.layer = 'systems'");
    const event = { clientX: 100, clientY: 100, pointerId: 1, target: { closest: () => null } };

    harness.handlers.get('#stage:pointerdown')(event);
    harness.handlers.get('#stage:pointercancel')(event);
    harness.handlers.get('#stage:pointerup')(event);
    harness.handlers.get('#stage:pointerdown')({ ...event, target: { closest: () => ({}) } });

    assert.equal(harness.evaluate('app.pointer.down'), false);
    assert.equal(harness.evaluate('app.selected'), null);
});

test('reduced motion overrides resume and unsafe public links are rejected', async () => {
    const harness = await browserHarness();

    harness.evaluate('app.running = true; app.reducedMotion = true; updateMotionButton()');

    assert.equal(harness.evaluate('app.running'), false);
    assert.equal(harness.evaluate("safePublicUrl('https://www.nuget.org/packages/example')"), true);
    assert.equal(harness.evaluate("safePublicUrl('javascript:alert(1)')"), false);
    assert.equal(harness.evaluate("safePublicUrl('https://user:password@example.com')"), false);
});

test('snapshot validation rejects corrupt geometry before active-state mutation', async () => {
    const harness = await browserHarness();
    harness.evaluate(`globalThis.validSnapshot = {
        nodes: [{id:'project:sample',label:'Sample',x:10,y:20}], edges: [],
        canvas: {width:1800,height:1680}, profile: {}, stats: {}, mode:'preview',
        semantic_hash:'0123456789abcdef'
    }`);

    const valid = harness.evaluate('validateState(validSnapshot).nodes.length');

    assert.equal(valid, 1);
    assert.throws(() => harness.evaluate('validateState({...validSnapshot, canvas:{width:NaN,height:1680}})'));
    assert.throws(() => harness.evaluate(
        'validateState({...validSnapshot, nodes:[...validSnapshot.nodes,...validSnapshot.nodes]})'));
    assert.throws(() => harness.evaluate(
        "validateState({...validSnapshot, edges:[{from:'missing',to:'project:sample',weight:1}]})"));
    assert.equal(harness.evaluate('app.state'), null);
});


test('private metrics distinguish a known zero from absent collection', async () => {
    const harness = await browserHarness();
    harness.evaluate("app.state = {stats:{private_repository_count:0},mode:'live'}; renderMetrics()");

    const collected = harness.elements.get('#metrics').children;
    const privateMetric = collected.find(item => item.children[0].includes('TOKEN-VISIBLE OWNED PRIVATE'));

    assert.ok(privateMetric);
    assert.equal(privateMetric.children[1].textContent, '0');

    harness.evaluate('app.state.stats.private_repository_count = null; renderMetrics()');

    assert.equal(harness.elements.get('#metrics').children.length, 4);

    harness.evaluate('delete app.state.stats.private_repository_count; renderMetrics()');

    assert.equal(harness.elements.get('#metrics').children.length, 4);
});


test('keyboard pause records user intent and respects reduced motion', async () => {
    const harness = await browserHarness();
    const event = {key:' ',code:'Space',target:harness.elements.get('body'),preventDefault() {}};

    harness.handlers.get('window:keydown')(event);

    assert.equal(harness.evaluate('app.running'), false);
    assert.equal(harness.evaluate('app.userPaused'), true);

    harness.evaluate('app.reducedMotion = true');
    harness.handlers.get('window:keydown')(event);

    assert.equal(harness.evaluate('app.running'), false);
    assert.equal(harness.evaluate('app.userPaused'), true);
});

test('profile animation timing preserves scan speed and safe signal phases', async () => {
    const harness = await browserHarness();
    const timings = [];
    const fixtures = [
        ['orbit', '', false], ['orbit', '', true], ['flow', '', false],
        ['signal', '-0.8', false], ['signal', '0', false],
        ['signal', 'url(https://example.com)', false], ['signal', 'Infinity', false],
        ['signal', '1', false], ['signal', '-1e309', false], ['signal', '', false],
    ];
    harness.context.svg = {
        getAnimations: () => fixtures.map(([animationName, signalDelay, scan]) => ({
            animationName,
            effect: {
                target: { dataset: { signalDelay }, classList: { contains: value => value === 'scan' && scan } },
                updateTiming: timing => timings.push({ ...timing }),
            },
        })),
    };

    harness.evaluate('configureProfileAnimations(svg, 32)');

    assert.deepEqual(timings.slice(0, 5), [
        { duration: 58000 }, { duration: 24000 }, { duration: 32000 },
        { duration: 3600, delay: -800 }, { duration: 3600, delay: 0 },
    ]);
    assert.ok(timings.slice(5).every(timing => timing.duration === 3600 && timing.delay === 0));
});

test('overview frames stay anchored while exploratory layers advance simulation', async () => {
    const harness = await browserHarness();
    let ticks = 0;
    let renders = 0;
    let decorationMoves = 0;
    harness.context.document.querySelectorAll = () => [{
        dataset: { nodeId: 'project:sample' },
        querySelector: () => ({ setAttribute: () => { decorationMoves++; } }),
    }];
    harness.context.requestAnimationFrame = () => {};
    harness.context.simulator = { tick: () => { ticks++; return [50, 60]; } };
    harness.context.renderer = { render: () => { renders++; } };
    harness.evaluate(`app.simulator = simulator; app.renderer = renderer;
        readPalette = () => ({}); buildVisualFrame = () => ({}); drawOverlay = () => {};
        app.positions = [10, 20];
        app.nodeById = new Map([['project:sample', {index:0, node:{x:5, y:5}}]]);`);

    harness.evaluate('frame(16)');

    assert.equal(ticks, 0);
    assert.equal(renders, 0);
    assert.equal(decorationMoves, 0);
    assert.deepEqual([...harness.evaluate('app.positions')], [10, 20]);

    harness.evaluate("app.layer = 'systems'; frame(32)");

    assert.equal(ticks, 1);
    assert.equal(renders, 1);
    assert.equal(decorationMoves, 0);
    assert.deepEqual([...harness.evaluate('app.positions')], [50, 60]);
});


test('leaving reduced motion reapplies phases to recreated animations without overriding user pause', async () => {
    for (const initiallyReduced of [false, true]) {
        const harness = await browserHarness({ reducedMotion: initiallyReduced });
        const timings = [];
        const svg = harness.context.document.querySelector('#profile-view svg');
        svg.getAnimations = () => [{
            animationName: 'signal',
            effect: {
                target: { dataset: { signalDelay: '-1.6' } },
                updateTiming: timing => timings.push({ ...timing }),
            },
        }];
        harness.evaluate('app.state = {canvas:{motion_seconds:48}}; app.userPaused = true');
        const change = harness.handlers.get('(prefers-reduced-motion: reduce):change');

        change({ matches: true });
        change({ matches: false });

        assert.deepEqual(timings, [{ duration: 3600, delay: -1600 }]);
        assert.equal(harness.evaluate('app.running'), false);
        assert.equal(harness.evaluate('app.userPaused'), true);
    }
});

test('resume and resize restore timing on newly created profile effects', async () => {
    const harness = await browserHarness();
    const timings = [];
    const svg = harness.context.document.querySelector('#profile-view svg');
    svg.getAnimations = () => [{
        animationName: 'signal',
        effect: {
            target: { dataset: { signalDelay: '-0.65' } },
            updateTiming: timing => timings.push({ ...timing }),
        },
    }];
    harness.evaluate('app.state = {canvas:{motion_seconds:32}}; app.renderer = {resize() {}}; app.running = false');

    harness.handlers.get('#motion-button:click')();
    harness.handlers.get('window:resize')();

    assert.deepEqual(timings, [
        { duration: 3600, delay: -650 },
        { duration: 3600, delay: -650 },
    ]);
    assert.equal(harness.evaluate('app.running'), true);
});
