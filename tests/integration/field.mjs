import assert from 'node:assert/strict';
import { join } from 'node:path';

/** Read actual SVG transforms and pulse opacity at deterministic animation times. */
async function checkMotion(page) {
    return page.evaluate(() => {
        const svg = document.querySelector('#profile-view svg');
        const animations = svg.getAnimations({ subtree: true });
        const rings = [...svg.querySelectorAll('[data-ring], [data-project-ring], [data-package-row]')];
        const originalTimes = animations.map(animation => animation.currentTime);

        /** Map the approved ring identity to clockwise (1) or counterclockwise (-1). */
        function expectedDirection(element) {
            if (element.dataset.ring) {
                return {
                    'personal-outer': 1,
                    'personal-inner': -1,
                    'organization-outer': -1,
                    'organization-inner': 1,
                }[element.dataset.ring];
            }

            if (element.dataset.projectRing) {
                return element.dataset.projectRing === 'middle' ? -1 : 1;
            }

            return element.dataset.packageRow === '1' ? -1 : 1;
        }

        /** Include ancestor transforms, so nested rotation cancellation is observable. */
        function angle(element) {
            const matrix = element.getCTM();
            return Math.atan2(matrix.b, matrix.a);
        }

        try {
            for (const animation of animations) {
                animation.currentTime = 1000;
            }

            const angles = rings.map(angle);
            for (const animation of animations) {
                animation.currentTime = 2000;
            }

            const directions = rings.map((element, index) => {
                const difference = angle(element) - angles[index];
                const direction = Math.sign(Math.atan2(Math.sin(difference), Math.cos(difference)));

                if (direction !== expectedDirection(element)) {
                    throw new Error(`Wrong direction: ${JSON.stringify(element.dataset)}`);
                }

                return { marker: { ...element.dataset }, direction };
            });
            const signals = animations.filter(animation => animation.animationName === 'signal');

            if (directions.length !== 26 || signals.length !== 11) {
                throw new Error(`Missing animation markers: ${directions.length} rings, ${signals.length} signals`);
            }

            const timing = signals.map(animation => {
                const effectTiming = animation.effect.getTiming();
                const declared = Number(animation.effect.target.dataset.signalDelay) * 1000;

                if (effectTiming.duration !== 3600 || Math.abs(effectTiming.delay - declared) > 0.001) {
                    throw new Error(`Lost signal timing: ${JSON.stringify({ timing: effectTiming, declared })}`);
                }

                // Sample a full cycle past the delay to avoid pre-active fill behavior.
                animation.currentTime = effectTiming.delay + 3600;
                const lo = Number(getComputedStyle(animation.effect.target).opacity);
                animation.currentTime = effectTiming.delay + 5220;
                const hi = Number(getComputedStyle(animation.effect.target).opacity);

                if (hi - lo < 0.7) {
                    throw new Error(`No signal pulse: ${JSON.stringify({ lo, hi, delay: effectTiming.delay })}`);
                }

                return { delay: effectTiming.delay, duration: effectTiming.duration, lo, hi };
            });
            const scan = animations.find(animation => animation.effect.target.classList.contains('scan'));

            if (scan?.effect.getTiming().duration !== 24000) {
                throw new Error('Wrong scan duration');
            }

            return {
                directions,
                timing,
                scanDuration: scan.effect.getTiming().duration,
                inlineStyles: svg.querySelectorAll('[style]').length,
            };
        } finally {
            animations.forEach((animation, index) => { animation.currentTime = originalTimes[index]; });
        }
    });
}

/** Read anchored SVG geometry independently of transient CSS animation transforms. */
async function geometry(page) {
    return page.evaluate(() => [...document.querySelectorAll(
        '#profile-view [data-node-decoration], #profile-view [data-edge-id] path, #profile-view [data-connection] path',
    )].map(element => [element.getAttribute('transform'), element.getAttribute('d')]));
}

/** Wait for paint frames rather than an arbitrary delay before comparing layout. */
async function paint(page) {
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
}

/** Check browser path geometry, radial tangency, evenly spaced ports, and package trunks. */
async function checkConnections(page) {
    const results = await page.evaluate(() => {
        const svg = document.querySelector('#profile-view svg');

        /** Measure SVG coordinates without depending on the viewport scale. */
        function distance(left, right) {
            return Math.hypot(left.x - right.x, left.y - right.y);
        }

        /** Read the visible outer ring as the attachment boundary. */
        function node(id) {
            const element = svg.querySelector(`[data-node-id="${id}"]`);
            return {
                x: Number(element.dataset.x),
                y: Number(element.dataset.y),
                r: Math.max(...[...element.querySelectorAll('[data-node-decoration] circle')]
                    .map(circle => Number(circle.getAttribute('r')))),
            };
        }

        let maxEndpointError = 0;
        let minAlignment = 1;
        const seen = new Set();
        const personalAngles = [];

        for (const path of svg.querySelectorAll('[data-edge-id] path, [data-connection="domain-bridge"] path')) {
            const parent = path.parentElement;
            const id = parent.dataset.edgeId || parent.dataset.connection;

            if (seen.has(id)) {
                continue;
            }

            seen.add(id);
            const source = node(id === 'domain-bridge' ? 'domain:personal' : parent.dataset.from);
            const target = node(id === 'domain-bridge' ? 'domain:doka-labs' : parent.dataset.to);
            const length = path.getTotalLength();
            const start = path.getPointAtLength(0);
            const end = path.getPointAtLength(length);

            if (id === 'domain-bridge' || parent.dataset.from === 'domain:personal') {
                personalAngles.push(Math.atan2(start.y - source.y, start.x - source.x) * 180 / Math.PI);
            }

            maxEndpointError = Math.max(
                maxEndpointError,
                Math.abs(distance(start, source) - source.r),
                Math.abs(distance(end, target) - target.r),
            );

            for (const [tip, near, center] of [
                [start, path.getPointAtLength(0.1), source],
                [end, path.getPointAtLength(length - 0.1), target],
            ]) {
                const tx = near.x - tip.x;
                const ty = near.y - tip.y;
                const rx = tip.x - center.x;
                const ry = tip.y - center.y;
                minAlignment = Math.min(minAlignment, (tx * rx + ty * ry) / (Math.hypot(tx, ty) * Math.hypot(rx, ry)));
            }

            for (let index = 0; index <= 500; index++) {
                const point = path.getPointAtLength(length * index / 500);

                if (distance(point, source) < source.r - 0.005 || distance(point, target) < target.r - 0.005) {
                    throw new Error(`Curve inside circle: ${id}`);
                }
            }
        }

        personalAngles.sort((left, right) => left - right);
        const personalGaps = personalAngles.map((angle, index) =>
            (personalAngles[(index + 1) % personalAngles.length] - angle + 360) % 360);

        if (personalAngles.length !== 6 || personalGaps.some(gap => Math.abs(gap - 60) > 0.001)) {
            throw new Error('Uneven private connection spacing');
        }

        const columns = [];

        for (const x of [85, 939]) {
            const paths = [...svg.querySelectorAll('.package-connector')]
                .filter(path => Number(path.getAttribute('d').match(/^M([0-9.]+)/)[1]) === x);
            const glyphs = [...svg.querySelectorAll('[data-node-kind="package"] g[transform]')]
                .filter(glyph => Number(glyph.transform.baseVal.getItem(0).matrix.e) === x);

            if (glyphs.length !== 3 || paths.length !== 3) {
                throw new Error(`Missing package column: ${x}`);
            }

            for (let index = 0; index < 3; index++) {
                const values = paths[index].getAttribute('d').match(/-?\d+(?:\.\d+)?/g).map(Number);
                const cy = glyphs[index].transform.baseVal.getItem(0).matrix.f;
                const radius = Number(glyphs[index].querySelector('circle').getAttribute('r'));

                if (values[0] !== x || values[2] !== cy - radius) {
                    throw new Error(`Wrong package segment end: ${x}/${index}`);
                }

                if (index > 0 && values[1] !== glyphs[index - 1].transform.baseVal.getItem(0).matrix.f + radius) {
                    throw new Error(`Wrong package segment start: ${x}/${index}`);
                }
            }

            columns.push(x);
        }

        return {
            curveCount: seen.size,
            personalPortAngles: personalAngles,
            personalPortGaps: personalGaps,
            maxEndpointError,
            minRadialAlignment: minAlignment,
            packageColumnCenters: columns,
            allCurvesOutsideCircles: true,
        };
    });

    assert.equal(results.curveCount, 9);
    assert.ok(results.maxEndpointError <= 0.005, JSON.stringify(results));
    assert.ok(results.minRadialAlignment >= 0.999, JSON.stringify(results));
    return results;
}

/**
 * Verify approved field geometry and animation behavior through theme and motion transitions.
 * @param {{browser: import('playwright').Browser, base: string, output: string}} options Runtime and artifact location.
 * @returns {Promise<object>} Motion, lifecycle, geometry, and browser error evidence.
 */
export async function runFieldChecks({ browser, base, output }) {
    const context = await browser.newContext({ viewport: { width: 1500, height: 1100 }, colorScheme: 'dark' });

    try {
        const page = await context.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(error.message));
        page.on('console', message => {
            if (message.type() === 'error') {
                errors.push(message.text());
            }
        });

        await page.goto(base);
        await page.waitForSelector('#profile-view svg');

        const result = { dark: await checkMotion(page), connections: await checkConnections(page) };
        const before = await geometry(page);

        await paint(page);

        assert.deepEqual(await geometry(page), before);
        await page.locator('[data-layer="systems"]').click();
        await paint(page);
        await page.locator('#stage').hover();
        await page.mouse.move(800, 450);
        await page.mouse.down();
        await page.mouse.move(850, 480);
        await page.mouse.up();
        await page.locator('[data-layer="overview"]').click();

        assert.deepEqual(await geometry(page), before);
        result.anchoredOverview = true;
        result.returnFromSystems = await checkMotion(page);
        await page.locator('#motion-button').click();

        assert.equal(await page.evaluate(() => document.querySelector('#profile-view svg')
            .getAnimations({ subtree: true }).every(animation => animation.playState === 'paused')), true);
        result.pause = true;
        await page.screenshot({ path: join(output, 'field-desktop-dark.png'), fullPage: true });

        const darkSvg = await page.locator('#profile-view svg').elementHandle();

        await page.emulateMedia({ colorScheme: 'light' });
        await darkSvg.waitForElementState('hidden');
        await darkSvg.dispose();
        await page.waitForSelector('#profile-view svg');
        await paint(page);

        assert.equal(await page.evaluate(() => document.querySelector('#profile-view svg')
            .getAnimations({ subtree: true }).every(animation => animation.playState === 'paused')), true);
        await page.screenshot({ path: join(output, 'field-desktop-light.png'), fullPage: true });
        await page.locator('#motion-button').click();
        result.light = await checkMotion(page);

        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.waitForFunction(() => document.querySelector('#motion-button').disabled);

        assert.equal(await page.locator('#motion-button').isDisabled(), true);
        assert.equal(await page.evaluate(() => document.querySelector('#profile-view svg')
            .getAnimations({ subtree: true }).filter(animation => animation.playState === 'running').length), 0);

        await page.reload();
        await page.waitForSelector('#profile-view svg');
        await page.emulateMedia({ reducedMotion: 'no-preference' });
        await page.waitForFunction(() => !document.querySelector('#motion-button').disabled);

        result.initialReducedRestoration = await checkMotion(page);
        await page.setViewportSize({ width: 390, height: 844 });
        await page.screenshot({ path: join(output, 'field-mobile.png'), fullPage: true });

        result.errors = errors;
        assert.deepEqual(errors, []);
        return result;
    } finally {
        await context.close();
    }
}
