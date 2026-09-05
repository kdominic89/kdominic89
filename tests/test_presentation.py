"""Geometric regressions for the approved, generated profile composition."""

import math
from pathlib import Path
import re
import unittest
import xml.etree.ElementTree as ET


ROOT = Path(__file__).resolve().parents[1]
NS = '{http://www.w3.org/2000/svg}'


def numbers(value):
    """Read generated path and transform coordinates without interpreting arbitrary SVG."""
    return [float(part) for part in re.findall(r'-?\d+(?:\.\d+)?', value)]


class PresentationTests(unittest.TestCase):
    """Test actual emitted boundaries and animation structure across all output variants."""

    def test_connection_boundaries_and_private_departures(self):
        for flavor in ('dark', 'light', 'static'):
            svg = ET.parse(ROOT / 'assets' / f'sourcefield.{flavor}.svg').getroot()
            nodes = {e.get('data-node-id'): e for e in svg.iter() if e.get('data-node-id')}
            pairs = [(g, g.get('data-from'), g.get('data-to')) for g in svg.iter() if g.get('data-edge-id')]
            bridge = next(g for g in svg.iter() if g.get('data-connection') == 'domain-bridge')
            pairs.append((bridge, 'domain:personal', 'domain:doka-labs'))
            angles = []

            self.assertEqual(len(pairs), 9)
            for group, source, target in pairs:
                centers = [(float(nodes[key].get('data-x')), float(nodes[key].get('data-y')))
                           for key in (source, target)]
                radii = [max(float(c.get('r')) for d in nodes[key].iter()
                             if d.get('data-node-decoration') for c in d.iter(f'{NS}circle'))
                         for key in (source, target)]
                paths = list(group)
                self.assertEqual(paths[0].get('d'), paths[1].get('d'))
                values = numbers(paths[0].get('d'))
                points = list(zip(values[::2], values[1::2]))
                self.assertEqual(len(points), 4)

                for tip, handle, center, radius in ((points[0], points[1], centers[0], radii[0]),
                                                    (points[3], points[2], centers[1], radii[1])):
                    self.assertAlmostEqual(math.dist(tip, center), radius, places=5)
                    ray = (tip[0] - center[0], tip[1] - center[1])
                    tangent = (handle[0] - tip[0], handle[1] - tip[1])
                    alignment = sum(a * b for a, b in zip(ray, tangent)) / (math.hypot(*ray) * math.hypot(*tangent))
                    self.assertGreater(alignment, .99999)

                for index in range(501):
                    t = index / 500
                    weights = ((1-t)**3, 3*(1-t)**2*t, 3*(1-t)*t*t, t**3)
                    point = tuple(sum(p[axis] * w for p, w in zip(points, weights)) for axis in (0, 1))
                    for center, radius in zip(centers, radii):
                        self.assertGreaterEqual(math.dist(point, center), radius - .00001)

                if source == 'domain:personal':
                    angles.append(math.atan2(points[0][1] - centers[0][1], points[0][0] - centers[0][0]))

            angles.sort()
            self.assertEqual(len(angles), 6)
            for index, angle in enumerate(angles):
                self.assertAlmostEqual((angles[(index + 1) % 6] - angle) % math.tau, math.tau / 6, places=6)

    def test_package_lines_end_on_rings_and_alternate_direction(self):
        svg = ET.parse(ROOT / 'assets/sourcefield.dark.svg').getroot()
        packages = [e for e in svg.iter() if e.get('data-node-kind') == 'package']
        lines = [numbers(e.get('d')) for e in svg.iter() if e.get('class') == 'package-connector']

        self.assertEqual(len(lines), 6)
        for index, package in enumerate(packages):
            glyph = next(e for e in package.iter() if e.get('transform', '').startswith('translate('))
            x, y = numbers(glyph.get('transform'))
            orbit = next(e for e in glyph if e.get('data-package-row') is not None)
            self.assertEqual(orbit.get('class'), 'rotate reverse' if index % 3 == 1 else 'rotate')
            line = next(line for line in lines if line[0] == x and line[2] == y - 16)
            self.assertLess(line[1], line[2])
            if index % 3:
                self.assertEqual(line[1], y - 80 + 16)

    def test_independent_rings_and_signal_inventory(self):
        svg = ET.parse(ROOT / 'assets/sourcefield.dark.svg').getroot()
        projects = [e for e in svg.iter() if e.get('data-node-kind') == 'project']

        for project in projects:
            decoration = next(e for e in project.iter() if e.get('data-node-decoration'))
            rings = {e.get('data-project-ring'): e for e in decoration if e.get('data-project-ring')}
            self.assertEqual(set(rings), {'outer', 'middle'})
            self.assertEqual(rings['outer'].get('class'), 'rotate')
            self.assertEqual(rings['middle'].get('class'), 'rotate reverse')

        for kind, count in (('personal-project', 5), ('nuget-package', 6)):
            signals = [e for e in svg.iter() if e.get('data-satellite') == kind]
            self.assertEqual(len(signals), count)
            self.assertEqual(len({e.get('data-signal-delay') for e in signals}), count)

        gradient = next(e for e in svg.iter() if e.get('id') == 'bridge-gradient')
        self.assertEqual([e.get('stop-color') for e in gradient], ['#4DE7C2', '#8B7CFF', '#D582FF', '#FFB86B'])
