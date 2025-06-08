// @ts-check

import { ComponentType } from "./enum";

/**
 * @typedef {import("../../types").CanvasComponent} CanvasComponent
 */

/**
 * @param {object} options
 * @param {HTMLCanvasElement} options.canvas
 * @param {number} options.aspect
 */
export function initDraw({ canvas, aspect }) {
  const ctx = /** @type {CanvasRenderingContext2D} */ (canvas.getContext("2d"));
  if (innerHeight * aspect > innerWidth) {
    canvas.width = innerWidth;
    canvas.height = innerWidth / aspect;
  } else {
    canvas.width = innerHeight * aspect;
    canvas.height = innerHeight;
  }
  /** @type {((components: CanvasComponent[]) => void)[]} */
  const componentProducers = [];
  return {
    getCanvas() {
      return canvas;
    },
    getContext() {
      return ctx;
    },
    adjustCanvas() {
      if (innerHeight * aspect > innerWidth) {
        canvas.width = innerWidth;
        canvas.height = innerWidth / aspect;
      } else {
        canvas.width = innerHeight * aspect;
        canvas.height = innerHeight;
      }
    },
    /**
     * @param  {...((components: CanvasComponent[]) => void)} producers
     */
    addComponentProducer(...producers) {
      for (const producer of producers) {
        componentProducers.push(producer);
      }
    },
    drawAll() {
      const components = componentProducers.reduce((acc, prod) => {
        prod(acc);
        return acc;
      }, /** @type {CanvasComponent[]} */ ([]));
      for (const component of components) {
        switch (component.type) {
          case ComponentType.Rect: {
            ctx.fillStyle = component.color;
            ctx.fillRect(
              // @ts-ignore
              ...[component.x, component.y, component.w, component.h].map(Math.round)
            );
            break;
          }
          case ComponentType.Text: {
            ctx.fillStyle = component.color;
            ctx.font = component.font;
            ctx.textAlign = component.textAlign;
            ctx.textBaseline = component.textBaseline;
            ctx.fillText(
              component.text,
              // @ts-ignore
              ...[component.x, component.y].map(Math.round)
            );
            break;
          }
          case ComponentType.Line: {
            ctx.strokeStyle = component.color;
            ctx.moveTo(
              // @ts-ignore
              ...[component.startX, component.startY].map(Math.round)
            );
            ctx.lineTo(
              // @ts-ignore
              ...[component.endX, component.endY].map(Math.round)
            );
            ctx.lineWidth = component.lineWidth;
            ctx.stroke();
            break;
          }
        }
      }
    },
  };
}
