// Spring easings shared by the capsule and the main window. Importing this
// module sets --yy-spring and --yy-soft on the document root; stylesheets
// should give a cubic-bezier fallback for engines without CSS linear().

export const reducedMotion =
  typeof window !== "undefined" &&
  window.matchMedia("(prefers-reduced-motion: reduce)").matches;

/** A CSS linear() easing sampled from a real damped spring. */
function springEasing(stiffness: number, damping: number): string {
  const samples: number[] = [];
  let x = 0;
  let v = 0;
  let t = 0;
  const dt = 1 / 240;
  while (t < 3) {
    v += (-stiffness * (x - 1) - damping * v) * dt;
    x += v * dt;
    t += dt;
    samples.push(x);
    if (t > 0.3 && Math.abs(x - 1) < 0.0006 && Math.abs(v) < 0.01) break;
  }
  const step = Math.max(1, Math.floor(samples.length / 50));
  const points: string[] = [];
  for (let i = 0; i < samples.length; i += step)
    points.push(samples[i].toFixed(4));
  return `linear(0, ${points.join(", ")}, 1)`;
}

if (
  !reducedMotion &&
  typeof CSS !== "undefined" &&
  CSS.supports("transition-timing-function", "linear(0, 1)")
) {
  const root = document.documentElement.style;
  root.setProperty("--yy-spring", springEasing(210, 24)); // a whisper of overshoot
  root.setProperty("--yy-soft", springEasing(150, 26)); // none
}
