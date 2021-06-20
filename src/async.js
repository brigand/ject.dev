import { EventType } from './EventType';

const measure = new EventType();
const render = new EventType();

let scheduled = false;
/**
 * @param {() => () => unknown} measureCb
 * @returns {() => undefined} cleanup
 * @example
 * queueMeasureRender(() => {
 *   const w = el.offsetWidth;
 *   return () => {
 *     el.style.height = w + 'px'
 *   };
 * });
 */
export function queueMeasureRender(measureCb) {
  let rmRender;
  const rmMeasure = measure.on(() => {
    scheduled = false;
    rmMeasure();
    const renderCb = measureCb();
    if (typeof renderCb === 'function') {
      rmRender = render.on(() => {
        rmRender();
        renderCb();
      });
    }
  });

  if (!scheduled) {
    const delay1 = window.setImmediate || window.setTimeout;

    delay1(() => {
      measure.emit(null);
      requestAnimationFrame(() => {
        render.emit(null);
      });
    });
  }

  const cleanup = () => {
    rmMeasure();
    if (rmRender) {
      rmRender();
    }
  };

  return cleanup;
}
