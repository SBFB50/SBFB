// anime.js v4.5 — definitions TypeScript canoniques (.d.ts)
// Agregees depuis dist/modules. Surface d'API publique exacte (types + signatures).
// 70 fichiers.


// ========================================================================
// adapters/index.d.ts
// ========================================================================
export { registerAdapter } from "./registry.js";

// ========================================================================
// adapters/registry.d.ts
// ========================================================================
/**
 * Creates and registers an Adapter. Each library extending `animate()` calls this once and uses the returned Adapter to wire up its target adapters and property resolvers. The optional `detect` short-circuits all lookups against the Adapter when the target is unrelated.
 *
 * @param {(t: any) => boolean} [detect]
 * @return {Adapter}
 */
export function registerAdapter(detect?: (t: any) => boolean): Adapter;
/**
 * Internal resolution. Tries every Adapter's target adapters first (in registration order, first match wins), then every Adapter's property resolvers.
 *
 * @param {any} target
 * @param {string} name
 * @return {TargetAdapterEntry | null}
 */
export function resolveAdapterEntry(target: any, name: string): TargetAdapterEntry | null;
export type TargetAdapterEntry = {
    get: (t: any) => any;
    set: (target: any, value: number, tween: any) => void;
    gate?: (t: any) => boolean;
};
declare class Adapter {
    /**
     * @param {((t: any) => boolean) | null} [detect]
     *   Optional gate. When provided, every lookup against this Adapter's target adapters and resolvers is skipped if `detect(target)` returns falsy. Lets the Adapter as a whole short-circuit on unrelated targets.
     */
    constructor(detect?: ((t: any) => boolean) | null);
    /** @type {((t: any) => boolean) | null} */
    detect: ((t: any) => boolean) | null;
    /** @type {TargetAdapter[]} */
    targetAdapters: TargetAdapter[];
    /** @type {((target: any, name: string) => TargetAdapterEntry | null)[]} */
    propertyResolvers: ((target: any, name: string) => TargetAdapterEntry | null)[];
    /**
     * Creates and registers a `TargetAdapter` scoped to this Adapter.
     *
     * @param {(t: any) => boolean} detect
     * @return {TargetAdapter}
     */
    registerTargetAdapter(detect: (t: any) => boolean): TargetAdapter;
    /**
     * Registers a property resolver scoped to this Adapter. Resolvers are functions invoked at tween creation when no target adapter has claimed the name; the function returns an entry for names it handles or `null` to defer. Use for runtime-matched patterns (Color / Vector axis detection, name-prefix conventions, etc.).
     *
     * @param {(target: any, name: string) => TargetAdapterEntry | null} resolver
     */
    registerPropertyResolver(resolver: (target: any, name: string) => TargetAdapterEntry | null): void;
}
declare class TargetAdapter {
    /**
     * @param {(t: any) => boolean} detect
     */
    constructor(detect: (t: any) => boolean);
    detect: (t: any) => boolean;
    /** @type {Record<string, TargetAdapterEntry>} */
    props: Record<string, TargetAdapterEntry>;
    /**
     * Registers a property the adapter handles. `setter` receives `(target, value, tween)`. For color and complex tweens `value` is `undefined`, read `tween._numbers` instead. `gate(target)` scopes the prop to a subset of matching targets.
     *
     * @param {string} name
     * @param {(t: any) => any} getter
     * @param {(target: any, value: number, tween: any) => void} setter
     * @param {(t: any) => boolean} [gate]
     */
    registerProperty(name: string, getter: (t: any) => any, setter: (target: any, value: number, tween: any) => void, gate?: (t: any) => boolean): void;
}
export {};

// ========================================================================
// adapters/three/adapter.d.ts
// ========================================================================
export const threeAdapter: {
    detect: (t: any) => boolean;
    targetAdapters: {
        detect: (t: any) => boolean;
        props: Record<string, import("../registry.js").TargetAdapterEntry>;
        registerProperty(name: string, getter: (t: any) => any, setter: (target: any, value: number, tween: any) => void, gate?: (t: any) => boolean): void;
    }[];
    propertyResolvers: ((target: any, name: string) => import("../registry.js").TargetAdapterEntry | null)[];
    registerTargetAdapter(detect: (t: any) => boolean): {
        detect: (t: any) => boolean;
        props: Record<string, import("../registry.js").TargetAdapterEntry>;
        registerProperty(name: string, getter: (t: any) => any, setter: (target: any, value: number, tween: any) => void, gate?: (t: any) => boolean): void;
    };
    registerPropertyResolver(resolver: (target: any, name: string) => TargetAdapterEntry | null): void;
};

// ========================================================================
// adapters/three/helpers.d.ts
// ========================================================================
/**
 * Patches a column-major `Matrix4.elements` array in place with CSS-style `skewX(α)` / `skewY(β)`, the 3D extension `skewZ(γ)` (shears z by x), and a `transform-origin` shift.
 *
 * @param {number[]} e
 * @param {number} skewX
 * @param {number} skewY
 * @param {number} skewZ
 * @param {number} ox
 * @param {number} oy
 * @param {number} oz
 */
export function applySkewOrigin(e: number[], skewX: number, skewY: number, skewZ: number, ox: number, oy: number, oz: number): void;
/**
 * @param {any} c
 * @return {String | null}
 */
export function readColorHex(c: any): string | null;
/**
 * Returns `true` when `v` is a `Vector2/3/4` instance whose dimension covers `axis`. `Quaternion` and plain `{x,y,z,w}`-shaped objects are intentionally rejected.
 *
 * @param {any} v
 * @param {'x' | 'y' | 'z' | 'w'} axis
 * @return {boolean}
 */
export function isVectorWith(v: any, axis: "x" | "y" | "z" | "w"): boolean;
/**
 * Classifies `target[name]` and returns a descriptor `{ kind, path, base, axis }` (or `null`). Used by the resolver entries in `resolvers.js` and by the `Object3D` adapter to build the matching access pattern for any three.js target (Object3D, Material, Texture, Fog, UniformNode).
 *
 * @param {any} target
 * @param {string} name
 * @return {{ kind: number, path: number, base: string, axis: 'x' | 'y' | 'z' | 'w' } | null}
 */
export function classifyTargetProp(target: any, name: string): {
    kind: number;
    path: number;
    base: string;
    axis: "x" | "y" | "z" | "w";
} | null;
/**
 * @param {any} target
 * @param {string} name
 * @param {number} path
 * @param {number} [defaultValue]
 * @return {number}
 */
export function readScalar(target: any, name: string, path: number, defaultValue?: number): number;
/**
 * @param {any} target
 * @param {string} name
 * @param {number} v
 * @param {number} path
 */
export function writeScalar(target: any, name: string, v: number, path: number): void;
/**
 * @param {any} target
 * @param {string} name
 * @param {number} path
 * @return {String | null}
 */
export function readColorAt(target: any, name: string, path: number): string | null;
/**
 * @param {any} target
 * @param {string} name
 * @param {Array.<Number>} ns
 * @param {number} path
 */
export function writeColorAt(target: any, name: string, ns: Array<number>, path: number): void;
/**
 * @param {any} target
 * @param {string} base
 * @param {'x' | 'y' | 'z' | 'w'} axis
 * @param {number} path
 * @return {number}
 */
export function readVectorAt(target: any, base: string, axis: "x" | "y" | "z" | "w", path: number): number;
/**
 * @param {any} target
 * @param {string} base
 * @param {'x' | 'y' | 'z' | 'w'} axis
 * @param {number} v
 * @param {number} path
 */
export function writeVectorAt(target: any, base: string, axis: "x" | "y" | "z" | "w", v: number, path: number): void;
export const COLOR_NORM: number;
export const AXIS_MAP: Record<string, "x" | "y" | "z" | "w">;
export const PATH_DIRECT: 0;
export const KIND_COLOR: 0;
export const KIND_SCALAR: 1;
export const KIND_VECTOR: 2;

// ========================================================================
// adapters/three/index.d.ts
// ========================================================================
export { threeAdapter } from "./adapter.js";
export { getInstances, commitChanges } from "./instance.js";

// ========================================================================
// adapters/three/instance.d.ts
// ========================================================================
/**
 * Flushes pending matrix writes for every dirty instance of `mesh`.
 * Called automatically before each render. Call it yourself if you read
 * `mesh.instanceMatrix` between an animation tick and the next render.
 *
 * @param {InstanceParent} mesh
 */
export function commitChanges(mesh: InstanceParent): void;
/**
 * Returns an array of per-instance adapters for `mesh`. Index by id, deleted slots on `BatchedMesh` are `null`. Pass the array (or a slice / a single element) to `animate()`.
 *
 * animate(getInstances(mesh), { x: 100, delay: stagger(5) });
 * animate(getInstances(mesh)[42], { scale: 2 });
 *
 * The same array reference is preserved across `mesh.count` / `addInstance` / `deleteInstance` calls. Entries are pushed, nulled, or truncated in place. Animations bound to an outdated reference keep tweening their original adapters.
 *
 * `mesh.onBeforeRender` is replaced with an accessor that flushes
 * pending instance writes before each render and forwards to your
 * handler. Assigning your own `mesh.onBeforeRender = fn` keeps the
 * auto-flush, but reading `mesh.onBeforeRender` afterwards returns the
 * chained dispatcher rather than `fn` itself, so identity checks
 * (`mesh.onBeforeRender === fn`) will not match.
 *
 * @param {InstanceParent} mesh
 * @return {(Instance | null)[]}
 */
export function getInstances(mesh: InstanceParent): (Instance | null)[];
export type InstanceParent = InstancedMesh | BatchedMesh;
/**
 * Per-instance adapter for `InstancedMesh` or `BatchedMesh`. Returned by `getInstances(mesh)`. Exposes the same flat properties as the mesh adapter, applied to a single instance id. Writes are coalesced and flushed before each render via `onBeforeRender`; call `commitChanges(mesh)` if you need to read `mesh.instanceMatrix` between a tick and a render.
 *
 * Caveats:
 * - `opacity` writes the parent's shared material, every instance is affected.
 * - `visible` is backed by `BatchedMesh.setVisibleAt`. On `InstancedMesh` it is a no-op, use `scale = 0` to hide an instance.
 */
declare class Instance {
    /**
     * @param {InstanceBinding} binding
     * @param {number} id
     */
    constructor(binding: InstanceBinding, id: number);
    isAnimejsInstanceProxy: boolean;
    parent: InstanceParent;
    id: number;
    _position: Vector3;
    _rotation: Euler;
    _scale: Vector3;
    _matrix: Matrix4;
    _quat: Quaternion;
    _color: Color;
    _dirty: number;
    _skewX: number;
    _skewY: number;
    _skewZ: number;
    _originX: number;
    _originY: number;
    _originZ: number;
    _hasSkewOrigin: boolean;
    /** @type {Instance[]} */
    _dirtyList: Instance[];
    _hasSetColor: boolean;
    _hasSetVisible: boolean;
    _hasGetVisible: boolean;
    /**
     * @param {number} flag
     */
    _markDirty(flag: number): void;
    _flush(): void;
    set x(v: number);
    get x(): number;
    set y(v: number);
    get y(): number;
    set z(v: number);
    get z(): number;
    set rotateX(v: number);
    get rotateX(): number;
    set rotateY(v: number);
    get rotateY(): number;
    set rotateZ(v: number);
    get rotateZ(): number;
    set scaleX(v: number);
    get scaleX(): number;
    set scaleY(v: number);
    get scaleY(): number;
    set scaleZ(v: number);
    get scaleZ(): number;
    set scale(v: number);
    get scale(): number;
    set skewX(v: number);
    get skewX(): number;
    set skewY(v: number);
    get skewY(): number;
    set skewZ(v: number);
    get skewZ(): number;
    set transformOriginX(v: number);
    get transformOriginX(): number;
    set transformOriginY(v: number);
    get transformOriginY(): number;
    set transformOriginZ(v: number);
    get transformOriginZ(): number;
    /** @param {number} v */
    set opacity(v: number);
    get opacity(): number;
    /** @param {boolean} v */
    set visible(v: boolean);
    get visible(): boolean;
}
import type { InstancedMesh } from 'three';
import { BatchedMesh } from 'three';
import { Vector3 } from 'three';
import { Euler } from 'three';
import { Matrix4 } from 'three';
import { Quaternion } from 'three';
import { Color } from 'three';
/**
 * Per-mesh state for `InstancedMesh` / `BatchedMesh` animations. Holds the instance array, the dirty queue, and the chained `onBeforeRender` closure.
 */
declare class InstanceBinding {
    /** @param {InstanceParent} mesh */
    constructor(mesh: InstanceParent);
    mesh: InstanceParent;
    hasInstanceMatrix: boolean;
    /** @type {(Instance | null)[]} */
    instances: (Instance | null)[];
    /** @type {Instance[]} */
    dirtyList: Instance[];
    /** @type {Object3D['onBeforeRender'] | null} */
    userOnBeforeRender: Object3D["onBeforeRender"] | null;
    chainedHandler: (renderer: import("three").WebGLRenderer, scene: import("three").Scene, camera: import("three").Camera, geometry: import("three").BufferGeometry, material: import("three").Material, group: import("three").Group) => void;
    flush(): void;
}
import type { Object3D } from 'three';
export {};

// ========================================================================
// adapters/three/object3d.d.ts
// ========================================================================
export {};

// ========================================================================
// adapters/three/resolvers.d.ts
// ========================================================================
export {};

// ========================================================================
// adapters/three/uniform.d.ts
// ========================================================================
export {};

// ========================================================================
// animatable/animatable.d.ts
// ========================================================================
/**
 * @import {
 * TargetsParam,
 * AnimatableParams,
 * AnimationParams,
 * TweenParamsOptions,
 * Tween,
 * AnimatableProperty,
 * AnimatableObject,
 * } from '../types/index.js';
 */
export class Animatable {
    /**
     * @param {TargetsParam} targets
     * @param {AnimatableParams} parameters
     */
    constructor(targets: TargetsParam, parameters: AnimatableParams);
    targets: (HTMLElement | SVGElement | import("../types/index.js").JSTarget)[];
    /** @type {Record<String, JSAnimation>} */
    animations: Record<string, JSAnimation>;
    /** @type {JSAnimation|null} */
    callbacks: JSAnimation | null;
    revert(): this;
}
export function createAnimatable(targets: TargetsParam, parameters: AnimatableParams): AnimatableObject;
import { JSAnimation } from '../animation/animation.js';
import type { TargetsParam } from '../types/index.js';
import type { AnimatableParams } from '../types/index.js';
import type { AnimatableObject } from '../types/index.js';

// ========================================================================
// animatable/index.d.ts
// ========================================================================
export * from "./animatable.js";

// ========================================================================
// animation/additive.d.ts
// ========================================================================
export namespace additive {
    export let animation: any;
    export { noop as update };
}
export function addAdditiveAnimation(lookups: TweenAdditiveLookups): AdditiveAnimation;
export type AdditiveAnimation = {
    duration: number;
    _offset: number;
    _delay: number;
    _head: Tween;
    _tail: Tween;
};
import { noop } from '../core/consts.js';
import type { TweenAdditiveLookups } from '../types/index.js';
import type { Tween } from '../types/index.js';

// ========================================================================
// animation/animation.d.ts
// ========================================================================
export class JSAnimation extends Timer {
    /**
     * @param {TargetsParam} targets
     * @param {AnimationParams} parameters
     * @param {Timeline} [parent]
     * @param {Number} [parentPosition]
     * @param {Boolean} [fastSet=false]
     * @param {Number} [index=0]
     * @param {TargetsArray} [allTargets]
     */
    constructor(targets: TargetsParam, parameters: AnimationParams, parent?: Timeline, parentPosition?: number, fastSet?: boolean, index?: number, allTargets?: TargetsArray);
    /** @type {Tween} */
    _head: Tween;
    /** @type {Tween} */
    _tail: Tween;
    /** @type {TargetsArray} */
    targets: TargetsArray;
    /** @type {Callback<this>} */
    onRender: Callback<this>;
    /** @type {EasingFunction} */
    _ease: EasingFunction;
    /**
     * @param  {Number} newDuration
     * @return {this}
     */
    stretch(newDuration: number): this;
    /**
     * @return {this}
     */
    refresh(): this;
    /**
     * Cancel the animation and revert all the values affected by this animation to their original state
     * @return {this}
     */
    revert(): this;
    /**
     * @typedef {this & {then: null}} ResolvedJSAnimation
     */
    /**
     * @param  {Callback<ResolvedJSAnimation>} [callback]
     * @return Promise<this>
     */
    then(callback?: Callback<this & {
        then: null;
    }>): Promise<any>;
}
export function animate(targets: TargetsParam, parameters: AnimationParams): JSAnimation;
import { Timer } from '../timer/timer.js';
import type { Tween } from '../types/index.js';
import type { TargetsArray } from '../types/index.js';
import type { Callback } from '../types/index.js';
import type { EasingFunction } from '../types/index.js';
import type { TargetsParam } from '../types/index.js';
import type { AnimationParams } from '../types/index.js';
import type { Timeline } from '../timeline/timeline.js';

// ========================================================================
// animation/composition.d.ts
// ========================================================================
export function getTweenSiblings(target: Target, property: string, lookup?: string): TweenPropertySiblings;
export function overrideTween(tween: Tween): void;
export function composeTween(tween: Tween, siblings: TweenPropertySiblings): Tween;
export function removeTweenSliblings(tween: Tween): Tween;
export function removeTargetsFromRenderable(targetsArray: TargetsArray, renderable?: Renderable, propertyName?: string): void;
import type { Target } from '../types/index.js';
import type { TweenPropertySiblings } from '../types/index.js';
import type { Tween } from '../types/index.js';
import type { TargetsArray } from '../types/index.js';
import type { Renderable } from '../types/index.js';

// ========================================================================
// animation/index.d.ts
// ========================================================================
export * from "./animation.js";

// ========================================================================
// core/clock.d.ts
// ========================================================================
/**
 * @import {
 *   Tickable,
 *   Tween,
 * } from '../types/index.js'
*/
export class Clock {
    /** @param {Number} [initTime] */
    constructor(initTime?: number);
    /** @type {Number} */
    deltaTime: number;
    /** @type {Number} */
    _currentTime: number;
    /** @type {Number} */
    _lastTickTime: number;
    /** @type {Number} */
    _startTime: number;
    /** @type {Number} */
    _lastTime: number;
    /** @type {Number} */
    _frameDuration: number;
    /** @type {Number} */
    _fps: number;
    /** @type {Number} */
    _speed: number;
    /** @type {Boolean} */
    _hasChildren: boolean;
    /** @type {Tickable|Tween} */
    _head: Tickable | Tween;
    /** @type {Tickable|Tween} */
    _tail: Tickable | Tween;
    set fps(frameRate: number);
    get fps(): number;
    set speed(playbackRate: number);
    get speed(): number;
    /**
     * @param  {Number} time
     * @return {tickModes}
     */
    requestTick(time: number): tickModes;
    /**
     * @param  {Number} time
     * @return {Number}
     */
    computeDeltaTime(time: number): number;
}
import type { Tickable } from '../types/index.js';
import type { Tween } from '../types/index.js';
import { tickModes } from './consts.js';

// ========================================================================
// core/colors.d.ts
// ========================================================================
export function convertColorStringValuesToRgbaArray(colorString: string): ColorArray;
import type { ColorArray } from '../types/index.js';

// ========================================================================
// core/consts.d.ts
// ========================================================================
export const isBrowser: boolean;
/** @typedef {Window & {AnimeJS: Array}|null} AnimeJSWindow

/** @type {AnimeJSWindow} */
export const win: AnimeJSWindow;
/** @type {Document|null} */
export const doc: Document | null;
export type tweenTypes = number;
export namespace tweenTypes {
    let OBJECT: number;
    let ATTRIBUTE: number;
    let CSS: number;
    let TRANSFORM: number;
    let CSS_VAR: number;
}
export type valueTypes = number;
export namespace valueTypes {
    let NUMBER: number;
    let UNIT: number;
    let COLOR: number;
    let COMPLEX: number;
}
export type tickModes = number;
export namespace tickModes {
    let NONE: number;
    let AUTO: number;
    let FORCE: number;
}
export type compositionTypes = number;
export namespace compositionTypes {
    let replace: number;
    let none: number;
    let blend: number;
}
export const isRegisteredTargetSymbol: unique symbol;
export const isDomSymbol: unique symbol;
export const isSvgSymbol: unique symbol;
export const transformsSymbol: unique symbol;
export const proxyTargetSymbol: unique symbol;
export const minValue: 1e-11;
export const maxValue: 1000000000000;
export const K: 1000;
export const maxFps: 240;
export const emptyString: "";
export const cssVarPrefix: "var(";
export const emptyArray: any[];
export const shortTransforms: Map<any, any>;
export const validTransforms: string[];
export const transformsFragmentStrings: {};
export function noop(): void;
export function noopModifier<T>(v: T): T;
export const validRgbHslRgx: RegExp;
export const hexTestRgx: RegExp;
export const rgbExecRgx: RegExp;
export const rgbaExecRgx: RegExp;
export const hslExecRgx: RegExp;
export const hslaExecRgx: RegExp;
export const digitWithExponentRgx: RegExp;
export const unitsExecRgx: RegExp;
export const lowerCaseRgx: RegExp;
export const relativeValuesExecRgx: RegExp;
export const cssVariableMatchRgx: RegExp;
/**
 * /**
 */
export type AnimeJSWindow = (Window & {
    AnimeJS: any[];
}) | null;

// ========================================================================
// core/globals.d.ts
// ========================================================================
/**
 * @import {
 *   DefaultsParams,
 *   DOMTarget,
 * } from '../types/index.js'
 *
 * @import {
 *   Scope,
 * } from '../scope/index.js'
*/
/**
 * @typedef {Object} EditorGlobals
 * @property {boolean} showPanel
 * @property {Function} addAnimation
 * @property {Function} addSet
 * @property {Function} addTimeline
 * @property {Function} addTimelineChild
 * @property {Function} addTimelineLabel
 * @property {Function} addTimelineCall
 * @property {Function} addTimelineSync
 * @property {Function} resolveStagger
 * @property {Object|null} _head
 * @property {Object|null} _tail
 */
/** @type {DefaultsParams} */
export const defaults: DefaultsParams;
export namespace scope {
    export let current: Scope;
    export { doc as root };
}
export namespace globals {
    export { defaults };
    export let precision: number;
    export let timeScale: number;
    export let tickThreshold: number;
    export let editor: EditorGlobals | null;
}
export namespace globalVersions {
    let version: string;
    let engine: any;
}
export type EditorGlobals = {
    showPanel: boolean;
    addAnimation: Function;
    addSet: Function;
    addTimeline: Function;
    addTimelineChild: Function;
    addTimelineLabel: Function;
    addTimelineCall: Function;
    addTimelineSync: Function;
    resolveStagger: Function;
    _head: any | null;
    _tail: any | null;
};
import type { DefaultsParams } from '../types/index.js';
import type { Scope } from '../scope/index.js';
import { doc } from './consts.js';

// ========================================================================
// core/helpers.d.ts
// ========================================================================
export function toLowerCase(str: string): string;
export function stringStartsWith(str: string, sub: string): boolean;
export const now: () => number;
export const isArr: (arg: any) => arg is any[];
export function isObj(a: any): a is Record<string, any>;
export function isNum(a: any): a is number;
export function isStr(a: any): a is string;
export function isFnc(a: any): a is Function;
export function isUnd(a: any): a is undefined;
export function isNil(a: any): a is null | undefined;
export function isSvg(a: any): a is SVGElement;
export function isHex(a: any): boolean;
export function isRgb(a: any): boolean;
export function isHsl(a: any): boolean;
export function isCol(a: any): boolean;
export function isKey(a: any): boolean;
export function isValidSVGAttribute(el: Target, propertyName: string): boolean;
export function parseNumber(str: number | string): number;
export const pow: (x: number, y: number) => number;
export const sqrt: (x: number) => number;
export const sin: (x: number) => number;
export const cos: (x: number) => number;
export const abs: (x: number) => number;
export const exp: (x: number) => number;
export const ceil: (x: number) => number;
export const floor: (x: number) => number;
export const asin: (x: number) => number;
export const max: (...values: number[]) => number;
export const atan2: (y: number, x: number) => number;
export const PI: number;
export const _round: (x: number) => number;
export function clamp(v: number, min: number, max: number): number;
export function round(v: number, decimalLength: number): number;
export function snap(v: number, increment: number | Array<number>): number;
export function lerp(start: number, end: number, factor: number): number;
export function clampInfinity(v: number): number;
export function normalizeTime(v: number): number;
export function cloneArray<T>(a: T[]): T[];
export function mergeObjects<T, U>(o1: T, o2: U): T & U;
export function forEachChildren(parent: any, callback: Function, reverse?: boolean, prevProp?: string, nextProp?: string): void;
export function removeChild(parent: any, child: any, prevProp?: string, nextProp?: string): void;
export function addChild(parent: any, child: any, sortMethod?: Function, prevProp?: string, nextProp?: string): void;
import type { Target } from '../types/index.js';

// ========================================================================
// core/render.d.ts
// ========================================================================
export function render(tickable: Tickable, time: number, muteCallbacks: number, internalRender: number, tickMode: tickModes): number;
export function tick(tickable: Tickable, time: number, muteCallbacks: number, internalRender: number, tickMode: number): void;
import type { Tickable } from '../types/index.js';
import { tickModes } from './consts.js';

// ========================================================================
// core/styles.d.ts
// ========================================================================
export function sanitizePropertyName(propertyName: string, target: Target, tweenType: tweenTypes): string;
export function revertValues<T extends Renderable>(renderable: T, inlineStylesOnly?: boolean): T;
export function cleanInlineStyles<T extends Renderable>(renderable: T): T;
import type { Target } from '../types/index.js';
import { tweenTypes } from './consts.js';
import type { Renderable } from '../types/index.js';

// ========================================================================
// core/targets.d.ts
// ========================================================================
/**
* @import {
*   DOMTarget,
*   DOMTargetsParam,
*   JSTargetsArray,
*   TargetsParam,
*   JSTargetsParam,
*   TargetsArray,
*   DOMTargetsArray,
* } from '../types/index.js'
*/
/**
 * @param  {DOMTargetsParam|TargetsParam} v
 * @return {NodeList|HTMLCollection}
 */
export function getNodeList(v: DOMTargetsParam | TargetsParam): NodeList | HTMLCollection;
/**
 * @overload
 * @param  {DOMTargetsParam} targets
 * @return {DOMTargetsArray}
 *
 * @overload
 * @param  {JSTargetsParam} targets
 * @return {JSTargetsArray}
 *
 * @overload
 * @param  {TargetsParam} targets
 * @return {TargetsArray}
 *
 * @param  {DOMTargetsParam|JSTargetsParam|TargetsParam} targets
 */
export function parseTargets(targets: DOMTargetsParam): DOMTargetsArray;
/**
 * @overload
 * @param  {DOMTargetsParam} targets
 * @return {DOMTargetsArray}
 *
 * @overload
 * @param  {JSTargetsParam} targets
 * @return {JSTargetsArray}
 *
 * @overload
 * @param  {TargetsParam} targets
 * @return {TargetsArray}
 *
 * @param  {DOMTargetsParam|JSTargetsParam|TargetsParam} targets
 */
export function parseTargets(targets: JSTargetsParam): JSTargetsArray;
/**
 * @overload
 * @param  {DOMTargetsParam} targets
 * @return {DOMTargetsArray}
 *
 * @overload
 * @param  {JSTargetsParam} targets
 * @return {JSTargetsArray}
 *
 * @overload
 * @param  {TargetsParam} targets
 * @return {TargetsArray}
 *
 * @param  {DOMTargetsParam|JSTargetsParam|TargetsParam} targets
 */
export function parseTargets(targets: TargetsParam): TargetsArray;
/**
 * @overload
 * @param  {DOMTargetsParam} targets
 * @return {DOMTargetsArray}
 *
 * @overload
 * @param  {JSTargetsParam} targets
 * @return {JSTargetsArray}
 *
 * @overload
 * @param  {TargetsParam} targets
 * @return {TargetsArray}
 *
 * @param  {DOMTargetsParam|JSTargetsParam|TargetsParam} targets
 */
export function registerTargets(targets: DOMTargetsParam): DOMTargetsArray;
/**
 * @overload
 * @param  {DOMTargetsParam} targets
 * @return {DOMTargetsArray}
 *
 * @overload
 * @param  {JSTargetsParam} targets
 * @return {JSTargetsArray}
 *
 * @overload
 * @param  {TargetsParam} targets
 * @return {TargetsArray}
 *
 * @param  {DOMTargetsParam|JSTargetsParam|TargetsParam} targets
 */
export function registerTargets(targets: JSTargetsParam): JSTargetsArray;
/**
 * @overload
 * @param  {DOMTargetsParam} targets
 * @return {DOMTargetsArray}
 *
 * @overload
 * @param  {JSTargetsParam} targets
 * @return {JSTargetsArray}
 *
 * @overload
 * @param  {TargetsParam} targets
 * @return {TargetsArray}
 *
 * @param  {DOMTargetsParam|JSTargetsParam|TargetsParam} targets
 */
export function registerTargets(targets: TargetsParam): TargetsArray;
import type { DOMTargetsParam } from '../types/index.js';
import type { TargetsParam } from '../types/index.js';
import type { DOMTargetsArray } from '../types/index.js';
import type { JSTargetsParam } from '../types/index.js';
import type { JSTargetsArray } from '../types/index.js';
import type { TargetsArray } from '../types/index.js';

// ========================================================================
// core/transforms.d.ts
// ========================================================================
export function parseInlineTransforms(target: DOMTarget, propName: string, animationInlineStyles: any): string;
export function buildTransformString(props: Record<string, string>): string;
import type { DOMTarget } from '../types/index.js';

// ========================================================================
// core/units.d.ts
// ========================================================================
export function convertValueUnit(el: DOMTarget, decomposedValue: TweenDecomposedValue, unit: string, force?: boolean): TweenDecomposedValue;
import type { DOMTarget } from '../types/index.js';
import type { TweenDecomposedValue } from '../types/index.js';

// ========================================================================
// core/values.d.ts
// ========================================================================
export function setValue<T, D>(targetValue: T | undefined, defaultValue: D): T | D;
export function getFunctionValue(value: TweenPropValue, target: Target, index: number, targets: TargetsArray, store: any | null, prevTween: Tween | null): any;
export function getTweenType(target: Target, prop: string): tweenTypes;
export function getOriginalAnimatableValue(target: Target, propName: string, tweenType?: tweenTypes, animationInlineStyles?: any | void): string | number;
export function getRelativeValue(x: number, y: number, operator: string): number;
export function createDecomposedValueTargetObject(): TweenDecomposedValue;
export function decomposeRawValue(rawValue: string | number | any, targetObject: TweenDecomposedValue): TweenDecomposedValue;
export function decomposeTweenValue(tween: Tween, targetObject: TweenDecomposedValue): TweenDecomposedValue;
export const decomposedOriginalValue: TweenDecomposedValue;
export function composeComplexValue(tween: Tween, progress: number, precision: number): string;
import type { TweenPropValue } from '../types/index.js';
import type { Target } from '../types/index.js';
import type { TargetsArray } from '../types/index.js';
import type { Tween } from '../types/index.js';
import { tweenTypes } from './consts.js';
import type { TweenDecomposedValue } from '../types/index.js';

// ========================================================================
// draggable/draggable.d.ts
// ========================================================================
export class Draggable {
    /**
     * @param {TargetsParam} target
     * @param {DraggableParams} [parameters]
     */
    constructor(target: TargetsParam, parameters?: DraggableParams);
    containerArray: number[];
    $container: HTMLElement;
    useWin: boolean;
    /** @type {Window | HTMLElement} */
    $scrollContainer: Window | HTMLElement;
    $target: HTMLElement;
    $trigger: HTMLElement;
    fixed: boolean;
    isFinePointer: boolean;
    /** @type {[Number, Number, Number, Number]} */
    containerPadding: [number, number, number, number];
    /** @type {Number} */
    containerFriction: number;
    /** @type {Number} */
    releaseContainerFriction: number;
    /** @type {Number|Array<Number>} */
    snapX: number | Array<number>;
    /** @type {Number|Array<Number>} */
    snapY: number | Array<number>;
    /** @type {Number} */
    scrollSpeed: number;
    /** @type {Number} */
    scrollThreshold: number;
    /** @type {Number} */
    dragSpeed: number;
    /** @type {Number} */
    dragThreshold: number;
    /** @type {Number} */
    maxVelocity: number;
    /** @type {Number} */
    minVelocity: number;
    /** @type {Number} */
    velocityMultiplier: number;
    /** @type {Boolean|DraggableCursorParams} */
    cursor: boolean | DraggableCursorParams;
    /** @type {Spring} */
    releaseXSpring: Spring;
    /** @type {Spring} */
    releaseYSpring: Spring;
    /** @type {EasingFunction} */
    releaseEase: EasingFunction;
    /** @type {Boolean} */
    hasReleaseSpring: boolean;
    /** @type {Callback<this>} */
    onGrab: Callback<this>;
    /** @type {Callback<this>} */
    onDrag: Callback<this>;
    /** @type {Callback<this>} */
    onRelease: Callback<this>;
    /** @type {Callback<this>} */
    onUpdate: Callback<this>;
    /** @type {Callback<this>} */
    onSettle: Callback<this>;
    /** @type {Callback<this>} */
    onSnap: Callback<this>;
    /** @type {Callback<this>} */
    onResize: Callback<this>;
    /** @type {Callback<this>} */
    onAfterResize: Callback<this>;
    /** @type {[Number, Number]} */
    disabled: [number, number];
    /** @type {AnimatableObject} */
    animate: AnimatableObject;
    xProp: string;
    yProp: string;
    destX: number;
    destY: number;
    deltaX: number;
    deltaY: number;
    scroll: {
        x: number;
        y: number;
    };
    /** @type {[Number, Number, Number, Number]} */
    coords: [number, number, number, number];
    /** @type {[Number, Number]} */
    snapped: [number, number];
    /** @type {[Number, Number, Number, Number, Number, Number, Number, Number]} */
    pointer: [number, number, number, number, number, number, number, number];
    /** @type {[Number, Number]} */
    scrollView: [number, number];
    /** @type {[Number, Number, Number, Number]} */
    dragArea: [number, number, number, number];
    /** @type {[Number, Number, Number, Number]} */
    containerBounds: [number, number, number, number];
    /** @type {[Number, Number, Number, Number]} */
    scrollBounds: [number, number, number, number];
    /** @type {[Number, Number, Number, Number]} */
    targetBounds: [number, number, number, number];
    /** @type {[Number, Number]} */
    window: [number, number];
    /** @type {[Number, Number, Number]} */
    velocityStack: [number, number, number];
    /** @type {Number} */
    velocityStackIndex: number;
    /** @type {Number} */
    velocityTime: number;
    /** @type {Number} */
    velocity: number;
    /** @type {Number} */
    angle: number;
    /** @type {JSAnimation} */
    cursorStyles: JSAnimation;
    /** @type {JSAnimation} */
    triggerStyles: JSAnimation;
    /** @type {JSAnimation} */
    bodyStyles: JSAnimation;
    /** @type {JSAnimation} */
    targetStyles: JSAnimation;
    /** @type {JSAnimation} */
    touchActionStyles: JSAnimation;
    transforms: Transforms;
    overshootCoords: {
        x: number;
        y: number;
    };
    overshootTicker: Timer;
    updated: boolean;
    manual: boolean;
    updateTicker: Timer;
    contained: boolean;
    grabbed: boolean;
    dragged: boolean;
    released: boolean;
    canScroll: boolean;
    enabled: boolean;
    initialized: boolean;
    activeProp: string;
    resizeTicker: Timer;
    parameters: DraggableParams;
    resizeObserver: ResizeObserver;
    /**
     * @param  {Number} dx
     * @param  {Number} dy
     * @return {Number}
     */
    computeVelocity(dx: number, dy: number): number;
    /**
     * @param {Number}  x
     * @param {Boolean} [muteUpdateCallback]
     * @return {this}
     */
    setX(x: number, muteUpdateCallback?: boolean): this;
    /**
     * @param {Number}  y
     * @param {Boolean} [muteUpdateCallback]
     * @return {this}
     */
    setY(y: number, muteUpdateCallback?: boolean): this;
    set x(x: number);
    get x(): number;
    set y(y: number);
    get y(): number;
    set progressX(x: number);
    get progressX(): number;
    set progressY(y: number);
    get progressY(): number;
    updateScrollCoords(): void;
    updateBoundingValues(): void;
    /**
     * @param  {Array} bounds
     * @param  {Number} x
     * @param  {Number} y
     * @return {Number}
     */
    isOutOfBounds(bounds: any[], x: number, y: number): number;
    refresh(): void;
    update(): void;
    stop(): this;
    /**
     * @param {Number} [duration]
     * @param {Number} [gap]
     * @param {EasingParam} [ease]
     * @return {this}
     */
    scrollInView(duration?: number, gap?: number, ease?: EasingParam): this;
    handleHover(): void;
    /**
     * @param  {Number} [duration]
     * @param  {Number} [gap]
     * @param  {EasingParam} [ease]
     * @return {this}
     */
    animateInView(duration?: number, gap?: number, ease?: EasingParam): this;
    /**
     * @param {MouseEvent|TouchEvent} e
     */
    handleDown(e: MouseEvent | TouchEvent): void;
    /**
     * @param {MouseEvent|TouchEvent} e
     */
    handleMove(e: MouseEvent | TouchEvent): void;
    handleUp(): void;
    reset(): this;
    enable(): this;
    disable(): this;
    revert(): this;
    /**
     * @param {Event} e
     */
    handleEvent(e: Event): void;
}
export function createDraggable(target: TargetsParam, parameters?: DraggableParams): Draggable;
import type { DraggableCursorParams } from '../types/index.js';
import type { Spring } from '../easings/spring/index.js';
import type { EasingFunction } from '../types/index.js';
import type { Callback } from '../types/index.js';
import type { AnimatableObject } from '../types/index.js';
import { JSAnimation } from '../animation/animation.js';
declare class Transforms {
    /**
     * @param {DOMTarget|DOMProxy} $el
     */
    constructor($el: DOMTarget | DOMProxy);
    $el: DOMTarget | DOMProxy;
    inlineTransforms: any[];
    point: DOMPoint;
    inversedMatrix: DOMMatrix;
    /**
     * @param {Number} x
     * @param {Number} y
     * @return {DOMPoint}
     */
    normalizePoint(x: number, y: number): DOMPoint;
    /**
     * @callback TraverseParentsCallback
     * @param {DOMTarget} $el
     * @param {Number} i
     */
    /**
     * @param {TraverseParentsCallback} cb
     */
    traverseUp(cb: ($el: DOMTarget, i: number) => any): void;
    getMatrix(): DOMMatrix;
    remove(): void;
    revert(): void;
}
import { Timer } from '../timer/timer.js';
import type { DraggableParams } from '../types/index.js';
import type { EasingParam } from '../types/index.js';
import type { TargetsParam } from '../types/index.js';
import type { DOMTarget } from '../types/index.js';
declare class DOMProxy {
    /** @param {Object} el */
    constructor(el: any);
    el: any;
    zIndex: number;
    parentElement: any;
    classList: {
        add: () => void;
        remove: () => void;
    };
    set x(v: any);
    get x(): any;
    set y(v: any);
    get y(): any;
    set width(v: any);
    get width(): any;
    set height(v: any);
    get height(): any;
    getBoundingClientRect(): {
        top: any;
        right: any;
        bottom: any;
        left: any;
    };
}
export {};

// ========================================================================
// draggable/index.d.ts
// ========================================================================
export * from "./draggable.js";

// ========================================================================
// easings/cubic-bezier/index.d.ts
// ========================================================================
export function cubicBezier(mX1?: number, mY1?: number, mX2?: number, mY2?: number): EasingFunction;
import type { EasingFunction } from '../../types/index.js';

// ========================================================================
// easings/eases/index.d.ts
// ========================================================================
export { eases } from "./parser.js";

// ========================================================================
// easings/eases/parser.d.ts
// ========================================================================
/**
 * @import {
 *   EasingFunction,
 *   EasingFunctionWithParams,
 *   EasingParam,
 *   BackEasing,
 *   ElasticEasing,
 *   PowerEasing,
 * } from '../../types/index.js'
*/
/** @type {PowerEasing} */
export const easeInPower: PowerEasing;
/**
 * @callback EaseType
 * @param {EasingFunction} Ease
 * @return {EasingFunction}
 */
/** @type {Record<String, EaseType>} */
export const easeTypes: Record<string, EaseType>;
export namespace eases {
    export let linear: EasingFunction;
    export let none: EasingFunction;
    let _in: PowerEasing;
    export { _in as in };
    export let out: PowerEasing;
    export let inOut: PowerEasing;
    export let outIn: PowerEasing;
    export let inQuad: EasingFunction;
    export let outQuad: EasingFunction;
    export let inOutQuad: EasingFunction;
    export let outInQuad: EasingFunction;
    export let inCubic: EasingFunction;
    export let outCubic: EasingFunction;
    export let inOutCubic: EasingFunction;
    export let outInCubic: EasingFunction;
    export let inQuart: EasingFunction;
    export let outQuart: EasingFunction;
    export let inOutQuart: EasingFunction;
    export let outInQuart: EasingFunction;
    export let inQuint: EasingFunction;
    export let outQuint: EasingFunction;
    export let inOutQuint: EasingFunction;
    export let outInQuint: EasingFunction;
    export let inSine: EasingFunction;
    export let outSine: EasingFunction;
    export let inOutSine: EasingFunction;
    export let outInSine: EasingFunction;
    export let inCirc: EasingFunction;
    export let outCirc: EasingFunction;
    export let inOutCirc: EasingFunction;
    export let outInCirc: EasingFunction;
    export let inExpo: EasingFunction;
    export let outExpo: EasingFunction;
    export let inOutExpo: EasingFunction;
    export let outInExpo: EasingFunction;
    export let inBounce: EasingFunction;
    export let outBounce: EasingFunction;
    export let inOutBounce: EasingFunction;
    export let outInBounce: EasingFunction;
    export let inBack: BackEasing;
    export let outBack: BackEasing;
    export let inOutBack: BackEasing;
    export let outInBack: BackEasing;
    export let inElastic: ElasticEasing;
    export let outElastic: ElasticEasing;
    export let inOutElastic: ElasticEasing;
    export let outInElastic: ElasticEasing;
}
export function parseEaseString(string: string): EasingFunction;
export function parseEase(ease: EasingParam): EasingFunction;
export type EaseType = (Ease: EasingFunction) => EasingFunction;
export type EasesFunctions = {
    linear: EasingFunction;
    none: EasingFunction;
    in: PowerEasing;
    out: PowerEasing;
    inOut: PowerEasing;
    outIn: PowerEasing;
    inQuad: EasingFunction;
    outQuad: EasingFunction;
    inOutQuad: EasingFunction;
    outInQuad: EasingFunction;
    inCubic: EasingFunction;
    outCubic: EasingFunction;
    inOutCubic: EasingFunction;
    outInCubic: EasingFunction;
    inQuart: EasingFunction;
    outQuart: EasingFunction;
    inOutQuart: EasingFunction;
    outInQuart: EasingFunction;
    inQuint: EasingFunction;
    outQuint: EasingFunction;
    inOutQuint: EasingFunction;
    outInQuint: EasingFunction;
    inSine: EasingFunction;
    outSine: EasingFunction;
    inOutSine: EasingFunction;
    outInSine: EasingFunction;
    inCirc: EasingFunction;
    outCirc: EasingFunction;
    inOutCirc: EasingFunction;
    outInCirc: EasingFunction;
    inExpo: EasingFunction;
    outExpo: EasingFunction;
    inOutExpo: EasingFunction;
    outInExpo: EasingFunction;
    inBounce: EasingFunction;
    outBounce: EasingFunction;
    inOutBounce: EasingFunction;
    outInBounce: EasingFunction;
    inBack: BackEasing;
    outBack: BackEasing;
    inOutBack: BackEasing;
    outInBack: BackEasing;
    inElastic: ElasticEasing;
    outElastic: ElasticEasing;
    inOutElastic: ElasticEasing;
    outInElastic: ElasticEasing;
};
import type { PowerEasing } from '../../types/index.js';
import type { EasingFunction } from '../../types/index.js';
import type { BackEasing } from '../../types/index.js';
import type { ElasticEasing } from '../../types/index.js';
import type { EasingParam } from '../../types/index.js';

// ========================================================================
// easings/index.d.ts
// ========================================================================
export * from "./cubic-bezier/index.js";
export * from "./steps/index.js";
export * from "./linear/index.js";
export * from "./irregular/index.js";
export * from "./spring/index.js";
export * from "./eases/index.js";

// ========================================================================
// easings/irregular/index.d.ts
// ========================================================================
export function irregular(length?: number, randomness?: number): EasingFunction;
import type { EasingFunction } from '../../types/index.js';

// ========================================================================
// easings/linear/index.d.ts
// ========================================================================
export function linear(...args: (string | number)[]): EasingFunction;
import type { EasingFunction } from '../../types/index.js';

// ========================================================================
// easings/none.d.ts
// ========================================================================
/**
 * @import {
 *   EasingFunction,
 * } from '../types/index.js'
*/
/** @type {EasingFunction} */
export const none: EasingFunction;
import type { EasingFunction } from '../types/index.js';

// ========================================================================
// easings/spring/index.d.ts
// ========================================================================
export class Spring {
    /**
     * @param {SpringParams} [parameters]
     */
    constructor(parameters?: SpringParams);
    timeStep: number;
    restThreshold: number;
    restDuration: number;
    maxDuration: number;
    maxRestSteps: number;
    maxIterations: number;
    bn: number;
    pd: number;
    m: number;
    s: number;
    d: number;
    v: number;
    w0: number;
    zeta: number;
    wd: number;
    b: number;
    completed: boolean;
    solverDuration: number;
    settlingDuration: number;
    /** @type {JSAnimation} */
    parent: JSAnimation;
    /** @type {Callback<JSAnimation>} */
    onComplete: Callback<JSAnimation>;
    /** @type {EasingFunction} */
    ease: EasingFunction;
    solve(time: number): number;
    calculateSDFromBD(): void;
    calculateBDFromSD(): void;
    compute(): void;
    set bounce(v: number);
    get bounce(): number;
    set duration(v: number);
    get duration(): number;
    set stiffness(v: number);
    get stiffness(): number;
    set damping(v: number);
    get damping(): number;
    set mass(v: number);
    get mass(): number;
    set velocity(v: number);
    get velocity(): number;
}
export function spring(parameters?: SpringParams): Spring;
export function createSpring(parameters?: SpringParams): Spring;
import type { JSAnimation } from '../../animation/animation.js';
import type { Callback } from '../../types/index.js';
import type { EasingFunction } from '../../types/index.js';
import type { SpringParams } from '../../types/index.js';

// ========================================================================
// easings/steps/index.d.ts
// ========================================================================
export function steps(steps?: number, fromStart?: boolean): EasingFunction;
import type { EasingFunction } from '../../types/index.js';

// ========================================================================
// engine/engine.d.ts
// ========================================================================
export const engine: Engine;
declare class Engine extends Clock {
    useDefaultMainLoop: boolean;
    pauseOnDocumentHidden: boolean;
    /** @type {DefaultsParams} */
    defaults: DefaultsParams;
    paused: boolean;
    /** @type {Number|NodeJS.Immediate} */
    reqId: number | NodeJS.Immediate;
    update(): void;
    wake(): this;
    pause(): Engine;
    resume(): this;
    set timeUnit(unit: "ms" | "s");
    get timeUnit(): "ms" | "s";
    set precision(precision: number);
    get precision(): number;
}
import { Clock } from '../core/clock.js';
import type { DefaultsParams } from '../types/index.js';
export {};

// ========================================================================
// engine/index.d.ts
// ========================================================================
export * from "./engine.js";

// ========================================================================
// events/index.d.ts
// ========================================================================
export * from "./scroll.js";

// ========================================================================
// events/scroll.d.ts
// ========================================================================
export const scrollContainers: Map<any, any>;
export class ScrollObserver {
    /**
     * @param {ScrollObserverParams} parameters
     */
    constructor(parameters?: ScrollObserverParams);
    /** @type {Number} */
    index: number;
    /** @type {String|Number} */
    id: string | number;
    /** @type {ScrollContainer} */
    container: ScrollContainer;
    /** @type {HTMLElement} */
    target: HTMLElement;
    /** @type {Tickable|WAAPIAnimation} */
    linked: Tickable | WAAPIAnimation;
    /** @type {Boolean} */
    repeat: boolean;
    /** @type {Boolean} */
    horizontal: boolean;
    /** @type {ScrollThresholdParam|ScrollThresholdValue|ScrollThresholdCallback} */
    enter: ScrollThresholdParam | ScrollThresholdValue | ScrollThresholdCallback;
    /** @type {ScrollThresholdParam|ScrollThresholdValue|ScrollThresholdCallback} */
    leave: ScrollThresholdParam | ScrollThresholdValue | ScrollThresholdCallback;
    /** @type {Boolean} */
    sync: boolean;
    /** @type {EasingFunction} */
    syncEase: EasingFunction;
    /** @type {Number} */
    syncSmooth: number;
    /** @type {Callback<ScrollObserver>} */
    onSyncEnter: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onSyncLeave: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onSyncEnterForward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onSyncLeaveForward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onSyncEnterBackward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onSyncLeaveBackward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onEnter: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onLeave: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onEnterForward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onLeaveForward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onEnterBackward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onLeaveBackward: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onUpdate: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onResize: Callback<ScrollObserver>;
    /** @type {Callback<ScrollObserver>} */
    onSyncComplete: Callback<ScrollObserver>;
    /** @type {Boolean} */
    reverted: boolean;
    /** @type {Boolean} */
    ready: boolean;
    /** @type {Boolean} */
    completed: boolean;
    /** @type {Boolean} */
    began: boolean;
    /** @type {Boolean} */
    isInView: boolean;
    /** @type {Boolean} */
    forceEnter: boolean;
    /** @type {Boolean} */
    hasEntered: boolean;
    /** @type {Number} */
    offset: number;
    /** @type {Number} */
    offsetStart: number;
    /** @type {Number} */
    offsetEnd: number;
    /** @type {Number} */
    distance: number;
    /** @type {Number} */
    prevProgress: number;
    /** @type {Array} */
    thresholds: any[];
    /** @type {[Number, Number, Number, Number]} */
    coords: [number, number, number, number];
    /** @type {JSAnimation} */
    debugStyles: JSAnimation;
    /** @type {HTMLElement} */
    $debug: HTMLElement;
    /** @type {ScrollObserverParams} */
    _params: ScrollObserverParams;
    /** @type {Boolean} */
    _debug: boolean;
    /** @type {ScrollObserver} */
    _next: ScrollObserver;
    /** @type {ScrollObserver} */
    _prev: ScrollObserver;
    /**
     * @param {Tickable|WAAPIAnimation} linked
     */
    link(linked: Tickable | WAAPIAnimation): this;
    get velocity(): number;
    get backward(): boolean;
    get scroll(): number;
    get progress(): number;
    refresh(): this;
    removeDebug(): this;
    debug(): void;
    updateBounds(): void;
    handleScroll(): void;
    revert(): this;
}
export function onScroll(parameters?: ScrollObserverParams): ScrollObserver;
declare class ScrollContainer {
    /**
     * @param {HTMLElement} $el
     */
    constructor($el: HTMLElement);
    /** @type {HTMLElement} */
    element: HTMLElement;
    /** @type {Boolean} */
    useWin: boolean;
    /** @type {Number} */
    winWidth: number;
    /** @type {Number} */
    winHeight: number;
    /** @type {Number} */
    width: number;
    /** @type {Number} */
    height: number;
    /** @type {Number} */
    left: number;
    /** @type {Number} */
    top: number;
    /** @type {Number} */
    scale: number;
    /** @type {Number} */
    zIndex: number;
    /** @type {Number} */
    scrollX: number;
    /** @type {Number} */
    scrollY: number;
    /** @type {Number} */
    prevScrollX: number;
    /** @type {Number} */
    prevScrollY: number;
    /** @type {Number} */
    scrollWidth: number;
    /** @type {Number} */
    scrollHeight: number;
    /** @type {Number} */
    velocity: number;
    /** @type {Boolean} */
    backwardX: boolean;
    /** @type {Boolean} */
    backwardY: boolean;
    /** @type {Timer} */
    scrollTicker: Timer;
    /** @type {Timer} */
    dataTimer: Timer;
    /** @type {Timer} */
    resizeTicker: Timer;
    /** @type {Timer} */
    wakeTicker: Timer;
    /** @type {ScrollObserver} */
    _head: ScrollObserver;
    /** @type {ScrollObserver} */
    _tail: ScrollObserver;
    resizeObserver: ResizeObserver;
    updateScrollCoords(): void;
    updateWindowBounds(): void;
    updateBounds(): void;
    refreshScrollObservers(): void;
    refresh(): void;
    handleScroll(): void;
    /**
     * @param {Event} e
     */
    handleEvent(e: Event): void;
    revert(): void;
}
import type { Tickable } from '../types/index.js';
import type { WAAPIAnimation } from '../waapi/waapi.js';
import type { ScrollThresholdParam } from '../types/index.js';
import type { ScrollThresholdValue } from '../types/index.js';
import type { ScrollThresholdCallback } from '../types/index.js';
import type { EasingFunction } from '../types/index.js';
import type { Callback } from '../types/index.js';
import type { JSAnimation } from '../animation/animation.js';
import type { ScrollObserverParams } from '../types/index.js';
import { Timer } from '../timer/timer.js';
export {};

// ========================================================================
// index.d.ts
// ========================================================================
export * from "./timer/index.js";
export * from "./animation/index.js";
export * from "./timeline/index.js";
export * from "./animatable/index.js";
export * from "./draggable/index.js";
export * from "./scope/index.js";
export * from "./events/index.js";
export * from "./engine/index.js";
export * from "./easings/index.js";
export * from "./layout/index.js";
export * from "./utils/index.js";
export * from "./svg/index.js";
export * from "./text/index.js";
export * from "./waapi/index.js";
export * from "./types/index.js";
export * as easings from "./easings/index.js";
export * as utils from "./utils/index.js";
export * as svg from "./svg/index.js";
export * as text from "./text/index.js";
export { globals } from "./core/globals.js";

// ========================================================================
// layout/index.d.ts
// ========================================================================
export * from "./layout.js";

// ========================================================================
// layout/layout.d.ts
// ========================================================================
export class AutoLayout {
    /**
     * @param {DOMTargetSelector} root
     * @param {AutoLayoutParams} [params]
     */
    constructor(root: DOMTargetSelector, params?: AutoLayoutParams);
    /** @type {AutoLayoutParams} */
    params: AutoLayoutParams;
    /** @type {DOMTarget} */
    root: DOMTarget;
    /** @type {Number|String} */
    id: number | string;
    /** @type {LayoutChildrenParam} */
    children: LayoutChildrenParam;
    /** @type {Boolean} */
    absoluteCoords: boolean;
    /** @type {LayoutStateParams} */
    swapAtParams: LayoutStateParams;
    /** @type {LayoutStateParams} */
    enterFromParams: LayoutStateParams;
    /** @type {LayoutStateParams} */
    leaveToParams: LayoutStateParams;
    /** @type {Set<String>} */
    properties: Set<string>;
    /** @type {Set<String>} */
    recordedProperties: Set<string>;
    /** @type {WeakSet<DOMTarget>} */
    pendingRemoval: WeakSet<DOMTarget>;
    /** @type {Map<DOMTarget, String|null>} */
    transitionMuteStore: Map<DOMTarget, string | null>;
    /** @type {LayoutSnapshot} */
    oldState: LayoutSnapshot;
    /** @type {LayoutSnapshot} */
    newState: LayoutSnapshot;
    /** @type {Timeline} */
    timeline: Timeline;
    /** @type {WAAPIAnimation} */
    transformAnimation: WAAPIAnimation;
    /** @type {Array<DOMTarget>} */
    animating: Array<DOMTarget>;
    /** @type {Array<DOMTarget>} */
    swapping: Array<DOMTarget>;
    /** @type {Array<DOMTarget>} */
    leaving: Array<DOMTarget>;
    /** @type {Array<DOMTarget>} */
    entering: Array<DOMTarget>;
    /**
     * @return {this}
     */
    revert(): this;
    /**
     * @return {this}
     */
    record(): this;
    /**
     * @param {LayoutAnimationParams} [params]
     * @return {Timeline}
     */
    animate(params?: LayoutAnimationParams): Timeline;
    /**
     * @param {(layout: this) => void} callback
     * @param {LayoutAnimationParams} [params]
     * @return {Timeline}
     */
    update(callback: (layout: this) => void, params?: LayoutAnimationParams): Timeline;
}
export function createLayout(root: DOMTargetSelector, params?: AutoLayoutParams): AutoLayout;
export type LayoutChildrenParam = DOMTargetSelector | Array<DOMTargetSelector>;
export type LayoutAnimationTimingsParams = {
    delay?: number | FunctionValue;
    duration?: number | FunctionValue;
    ease?: EasingParam | FunctionValue;
};
export type LayoutStateAnimationProperties = Record<string, number | string | FunctionValue>;
export type LayoutStateParams = LayoutStateAnimationProperties & LayoutAnimationTimingsParams;
export type LayoutSpecificAnimationParams = {
    id?: number | string;
    delay?: number | FunctionValue;
    duration?: number | FunctionValue;
    ease?: EasingParam | FunctionValue;
    playbackEase?: EasingParam;
    swapAt?: LayoutStateParams;
    enterFrom?: LayoutStateParams;
    leaveTo?: LayoutStateParams;
};
export type LayoutAnimationParams = LayoutSpecificAnimationParams & TimerParams & TickableCallbacks<Timeline> & RenderableCallbacks<Timeline>;
export type LayoutOptions = {
    children?: LayoutChildrenParam;
    properties?: Array<string>;
};
export type AutoLayoutParams = LayoutAnimationParams & LayoutOptions;
export type LayoutNodeProperties = Record<string, number | string | FunctionValue> & {
    transform: string;
    x: number;
    y: number;
    left: number;
    top: number;
    clientLeft: number;
    clientTop: number;
    width: number;
    height: number;
};
export type LayoutNode = {
    id: string;
    $el: DOMTarget;
    index: number;
    targets: Array<DOMTarget>;
    delay: number;
    duration: number;
    ease: EasingParam;
    $measure: DOMTarget;
    state: LayoutSnapshot;
    layout: AutoLayout;
    parentNode: LayoutNode | null;
    isTarget: boolean;
    isEntering: boolean;
    isLeaving: boolean;
    hasTransform: boolean;
    inlineStyles: Array<string>;
    inlineTransforms: string | null;
    inlineTransition: string | null;
    branchAdded: boolean;
    branchRemoved: boolean;
    branchNotRendered: boolean;
    sizeChanged: boolean;
    isInlined: boolean;
    hasVisibilitySwap: boolean;
    hasDisplayNone: boolean;
    hasVisibilityHidden: boolean;
    measuredInlineTransform: string | null;
    measuredInlineTransition: string | null;
    measuredDisplay: string | null;
    measuredVisibility: string | null;
    measuredPosition: string | null;
    measuredHasDisplayNone: boolean;
    measuredHasVisibilityHidden: boolean;
    measuredIsVisible: boolean;
    measuredIsRemoved: boolean;
    measuredIsInsideRoot: boolean;
    properties: LayoutNodeProperties;
    _head: LayoutNode | null;
    _tail: LayoutNode | null;
    _prev: LayoutNode | null;
    _next: LayoutNode | null;
};
export type LayoutNodeIterator = (node: LayoutNode, index: number) => void;
import type { DOMTarget } from '../types/index.js';
declare class LayoutSnapshot {
    /**
     * @param {AutoLayout} layout
     */
    constructor(layout: AutoLayout);
    /** @type {AutoLayout} */
    layout: AutoLayout;
    /** @type {LayoutNode|null} */
    rootNode: LayoutNode | null;
    /** @type {Set<LayoutNode>} */
    rootNodes: Set<LayoutNode>;
    /** @type {Map<String, LayoutNode>} */
    nodes: Map<string, LayoutNode>;
    /** @type {Number} */
    scrollX: number;
    /** @type {Number} */
    scrollY: number;
    /**
     * @return {this}
     */
    revert(): this;
    /**
     * @param {DOMTarget} $el
     * @return {LayoutNode}
     */
    getNode($el: DOMTarget): LayoutNode;
    /**
     * @param {DOMTarget} $el
     * @param {String} prop
     * @return {Number|String}
     */
    getComputedValue($el: DOMTarget, prop: string): number | string;
    /**
     * @param {LayoutNode|null} rootNode
     * @param {LayoutNodeIterator} cb
     */
    forEach(rootNode: LayoutNode | null, cb: LayoutNodeIterator): void;
    /**
     * @param {LayoutNodeIterator} cb
     */
    forEachRootNode(cb: LayoutNodeIterator): void;
    /**
     * @param {LayoutNodeIterator} cb
     */
    forEachNode(cb: LayoutNodeIterator): void;
    /**
     * @param {DOMTarget} $el
     * @param {LayoutNode|null} parentNode
     * @return {LayoutNode|null}
     */
    registerElement($el: DOMTarget, parentNode: LayoutNode | null): LayoutNode | null;
    /**
     * @param {DOMTarget} $el
     * @param {Set<DOMTarget>} candidates
     * @return {LayoutNode|null}
     */
    ensureDetachedNode($el: DOMTarget, candidates: Set<DOMTarget>): LayoutNode | null;
    /**
     * @return {this}
     */
    record(): this;
}
import type { Timeline } from '../timeline/timeline.js';
import type { WAAPIAnimation } from '../waapi/waapi.js';
import type { DOMTargetSelector } from '../types/index.js';
import type { FunctionValue } from '../types/index.js';
import type { EasingParam } from '../types/index.js';
import type { TimerParams } from '../types/index.js';
import type { TickableCallbacks } from '../types/index.js';
import type { RenderableCallbacks } from '../types/index.js';
export {};

// ========================================================================
// scope/index.d.ts
// ========================================================================
export * from "./scope.js";

// ========================================================================
// scope/scope.d.ts
// ========================================================================
/**
 * @import {
 *   Tickable,
 *   ScopeParams,
 *   DOMTarget,
 *   ReactRef,
 *   AngularRef,
 *   DOMTargetSelector,
 *   DefaultsParams,
 *   ScopeConstructorCallback,
 *   ScopeCleanupCallback,
 *   Revertible,
 *   ScopeMethod,
 *   ScopedCallback,
 * } from '../types/index.js'
*/
export class Scope {
    /** @param {ScopeParams} [parameters] */
    constructor(parameters?: ScopeParams);
    /** @type {DefaultsParams} */
    defaults: DefaultsParams;
    /** @type {Document|DOMTarget} */
    root: Document | DOMTarget;
    /** @type {Array<ScopeConstructorCallback>} */
    constructors: Array<ScopeConstructorCallback>;
    /** @type {Array<ScopeCleanupCallback>} */
    revertConstructors: Array<ScopeCleanupCallback>;
    /** @type {Array<Revertible>} */
    revertibles: Array<Revertible>;
    /** @type {Array<ScopeConstructorCallback | ((scope: this) => Tickable)>} */
    constructorsOnce: Array<ScopeConstructorCallback | ((scope: this) => Tickable)>;
    /** @type {Array<ScopeCleanupCallback>} */
    revertConstructorsOnce: Array<ScopeCleanupCallback>;
    /** @type {Array<Revertible>} */
    revertiblesOnce: Array<Revertible>;
    /** @type {Boolean} */
    once: boolean;
    /** @type {Number} */
    onceIndex: number;
    /** @type {Record<String, ScopeMethod>} */
    methods: Record<string, ScopeMethod>;
    /** @type {Record<String, Boolean>} */
    matches: Record<string, boolean>;
    /** @type {Record<String, MediaQueryList>} */
    mediaQueryLists: Record<string, MediaQueryList>;
    /** @type {Record<String, any>} */
    data: Record<string, any>;
    /**
     * @param {Revertible} revertible
     */
    register(revertible: Revertible): void;
    /**
     * @template T
     * @param {ScopedCallback<T>} cb
     * @return {T}
     */
    execute<T>(cb: ScopedCallback<T>): T;
    /**
     * @return {this}
     */
    refresh(): this;
    /**
     * @overload
     * @param {String} a1
     * @param {ScopeMethod} a2
     * @return {this}
     *
     * @overload
     * @param {ScopeConstructorCallback} a1
     * @return {this}
     *
     * @param {String|ScopeConstructorCallback} a1
     * @param {ScopeMethod} [a2]
     */
    add(a1: string, a2: ScopeMethod): this;
    /**
     * @overload
     * @param {String} a1
     * @param {ScopeMethod} a2
     * @return {this}
     *
     * @overload
     * @param {ScopeConstructorCallback} a1
     * @return {this}
     *
     * @param {String|ScopeConstructorCallback} a1
     * @param {ScopeMethod} [a2]
     */
    add(a1: ScopeConstructorCallback): this;
    /**
     * @param {ScopeConstructorCallback} scopeConstructorCallback
     * @return {this}
     */
    addOnce(scopeConstructorCallback: ScopeConstructorCallback): this;
    /**
     * @param  {(scope: this) => Tickable} cb
     * @return {Tickable}
     */
    keepTime(cb: (scope: this) => Tickable): Tickable;
    /**
     * @param {Event} e
     */
    handleEvent(e: Event): void;
    revert(): void;
}
export function createScope(params?: ScopeParams): Scope;
import type { DefaultsParams } from '../types/index.js';
import type { DOMTarget } from '../types/index.js';
import type { ScopeConstructorCallback } from '../types/index.js';
import type { ScopeCleanupCallback } from '../types/index.js';
import type { Revertible } from '../types/index.js';
import type { Tickable } from '../types/index.js';
import type { ScopeMethod } from '../types/index.js';
import type { ScopedCallback } from '../types/index.js';
import type { ScopeParams } from '../types/index.js';

// ========================================================================
// svg/drawable.d.ts
// ========================================================================
export function createDrawable(selector: TargetsParam, start?: number, end?: number): Array<DrawableSVGGeometry>;
import type { TargetsParam } from '../types/index.js';
import type { DrawableSVGGeometry } from '../types/index.js';

// ========================================================================
// svg/helpers.d.ts
// ========================================================================
export function getPath(path: TargetsParam): SVGGeometryElement | void;
import type { TargetsParam } from '../types/index.js';

// ========================================================================
// svg/index.d.ts
// ========================================================================
export { createMotionPath } from "./motionpath.js";
export { createDrawable } from "./drawable.js";
export { morphTo } from "./morphto.js";

// ========================================================================
// svg/morphto.d.ts
// ========================================================================
export function morphTo(path2: TargetsParam, precision?: number): FunctionValue;
import type { TargetsParam } from '../types/index.js';
import type { FunctionValue } from '../types/index.js';

// ========================================================================
// svg/motionpath.d.ts
// ========================================================================
export function createMotionPath(path: TargetsParam, offset?: number): {
    translateX: FunctionValue;
    translateY: FunctionValue;
    rotate: FunctionValue;
};
import type { TargetsParam } from '../types/index.js';
import type { FunctionValue } from '../types/index.js';

// ========================================================================
// text/index.d.ts
// ========================================================================
export * from "./split.js";
export * from "./scramble.js";

// ========================================================================
// text/scramble.d.ts
// ========================================================================
export function scrambleText(params?: ScrambleTextParams): FunctionValue<ScrambleTextTween>;
export type ScrambleTextTween = {
    from: number;
    to: number;
    duration: number;
    delay: number;
    ease: string;
    modifier: (v: number) => string;
};
import type { ScrambleTextParams } from '../types/index.js';
import type { FunctionValue } from '../types/index.js';

// ========================================================================
// text/split.d.ts
// ========================================================================
/**
 * A class that splits text into words and wraps them in span elements while preserving the original HTML structure.
 * @class
 */
export class TextSplitter {
    /**
     * @param  {Element|NodeList|String|Array<Element>} target
     * @param  {TextSplitterParams} [parameters]
     */
    constructor(target: Element | NodeList | string | Array<Element>, parameters?: TextSplitterParams);
    debug: boolean;
    includeSpaces: boolean;
    accessible: boolean;
    linesOnly: boolean;
    /** @type {String|false|SplitFunctionValue} */
    lineTemplate: string | false | SplitFunctionValue;
    /** @type {String|false|SplitFunctionValue} */
    wordTemplate: string | false | SplitFunctionValue;
    /** @type {String|false|SplitFunctionValue} */
    charTemplate: string | false | SplitFunctionValue;
    $target: HTMLElement;
    html: string;
    lines: any[];
    words: any[];
    chars: any[];
    effects: any[];
    effectsCleanups: any[];
    cache: string;
    ready: boolean;
    width: number;
    resizeTimeout: NodeJS.Timeout;
    resizeObserver: ResizeObserver;
    /**
     * @param  {(...args: any[]) => Tickable | (() => void) | void} effect
     * @return this
     */
    addEffect(effect: (...args: any[]) => Tickable | (() => void) | void): this;
    revert(): this;
    /**
     * Recursively processes a node and its children
     * @param {Node} node
     */
    splitNode(node: Node): void;
    /**
     * @param {Boolean} clearCache
     * @return {this}
     */
    split(clearCache?: boolean): this;
    refresh(): void;
}
export function splitText(target: Element | NodeList | string | Array<Element>, parameters?: TextSplitterParams): TextSplitter;
export function split(target: HTMLElement | NodeList | string | Array<HTMLElement>, parameters?: TextSplitterParams): TextSplitter;
export type Segment = {
    segment: string;
    isWordLike?: boolean;
};
export type Segmenter = {
    segment: (arg0: string) => Iterable<Segment>;
};
import type { SplitFunctionValue } from '../types/index.js';
import type { Tickable } from '../types/index.js';
import type { TextSplitterParams } from '../types/index.js';

// ========================================================================
// timeline/index.d.ts
// ========================================================================
export * from "./timeline.js";

// ========================================================================
// timeline/position.d.ts
// ========================================================================
export function parseTimelinePosition(timeline: Timeline, timePosition?: TimelinePosition): number;
import type { Timeline } from './timeline.js';
import type { TimelinePosition } from '../types/index.js';

// ========================================================================
// timeline/timeline.d.ts
// ========================================================================
export class Timeline extends Timer {
    /**
     * @param {TimelineParams} [parameters]
     */
    constructor(parameters?: TimelineParams);
    /** @type {Record<String, Number>} */
    labels: Record<string, number>;
    /** @type {DefaultsParams} */
    defaults: DefaultsParams;
    /** @type {Boolean} */
    composition: boolean;
    /** @type {Callback<this>} */
    onRender: Callback<this>;
    _ease: import("../types/index.js").EasingFunction;
    /**
     * @overload
     * @param {TargetsParam} a1
     * @param {AnimationParams} a2
     * @param {TimelinePosition|StaggerFunction<Number|String>|TweakRegister} [a3]
     * @return {this}
     *
     * @overload
     * @param {TimerParams} a1
     * @param {TimelinePosition} [a2]
     * @return {this}
     *
     * @param {TargetsParam|TimerParams} a1
     * @param {TimelinePosition|AnimationParams} a2
     * @param {TimelinePosition|StaggerFunction<Number|String>|TweakRegister} [a3]
     */
    add(a1: TargetsParam, a2: AnimationParams, a3?: TimelinePosition | StaggerFunction<number | string> | TweakRegister): this;
    /**
     * @overload
     * @param {TargetsParam} a1
     * @param {AnimationParams} a2
     * @param {TimelinePosition|StaggerFunction<Number|String>|TweakRegister} [a3]
     * @return {this}
     *
     * @overload
     * @param {TimerParams} a1
     * @param {TimelinePosition} [a2]
     * @return {this}
     *
     * @param {TargetsParam|TimerParams} a1
     * @param {TimelinePosition|AnimationParams} a2
     * @param {TimelinePosition|StaggerFunction<Number|String>|TweakRegister} [a3]
     */
    add(a1: TimerParams, a2?: TimelinePosition): this;
    /**
     * @overload
     * @param {Tickable} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @overload
     * @param {globalThis.Animation} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @overload
     * @param {WAAPIAnimation} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @param {Tickable|WAAPIAnimation|globalThis.Animation} [synced]
     * @param {TimelinePosition} [position]
     */
    sync(synced?: Tickable, position?: TimelinePosition): this;
    /**
     * @overload
     * @param {Tickable} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @overload
     * @param {globalThis.Animation} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @overload
     * @param {WAAPIAnimation} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @param {Tickable|WAAPIAnimation|globalThis.Animation} [synced]
     * @param {TimelinePosition} [position]
     */
    sync(synced?: globalThis.Animation, position?: TimelinePosition): this;
    /**
     * @overload
     * @param {Tickable} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @overload
     * @param {globalThis.Animation} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @overload
     * @param {WAAPIAnimation} [synced]
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     * @param {Tickable|WAAPIAnimation|globalThis.Animation} [synced]
     * @param {TimelinePosition} [position]
     */
    sync(synced?: WAAPIAnimation, position?: TimelinePosition): this;
    /**
     * @param  {TargetsParam} targets
     * @param  {AnimationParams} parameters
     * @param  {TimelinePosition|StaggerFunction<Number|String>|TweakRegister} [position]
     * @return {this}
     */
    set(targets: TargetsParam, parameters: AnimationParams, position?: TimelinePosition | StaggerFunction<number | string> | TweakRegister): this;
    /**
     * @param {Callback<Timer>} callback
     * @param {TimelinePosition} [position]
     * @return {this}
     */
    call(callback: Callback<Timer>, position?: TimelinePosition): this;
    /**
     * @param {String} labelName
     * @param {TimelinePosition} [position]
     * @return {this}
     *
     */
    label(labelName: string, position?: TimelinePosition): this;
    /**
     * @param  {TargetsParam} targets
     * @param  {String} [propertyName]
     * @return {this}
     */
    remove(targets: TargetsParam, propertyName?: string): this;
    /**
     * @param  {Number} newDuration
     * @return {this}
     */
    stretch(newDuration: number): this;
    /**
     * @return {this}
     */
    refresh(): this;
    /**
     * @return {this}
     */
    revert(): this;
    /**
     * @typedef {this & {then: null}} ResolvedTimeline
     */
    /**
     * @param  {Callback<ResolvedTimeline>} [callback]
     * @return Promise<this>
     */
    then(callback?: Callback<this & {
        then: null;
    }>): Promise<any>;
}
export function createTimeline(parameters?: TimelineParams): Timeline;
import { Timer } from '../timer/timer.js';
import type { DefaultsParams } from '../types/index.js';
import type { Callback } from '../types/index.js';
import type { TargetsParam } from '../types/index.js';
import type { AnimationParams } from '../types/index.js';
import type { TimelinePosition } from '../types/index.js';
import type { StaggerFunction } from '../types/index.js';
import type { TweakRegister } from '../types/index.js';
import type { TimerParams } from '../types/index.js';
import type { Tickable } from '../types/index.js';
import type { WAAPIAnimation } from '../waapi/waapi.js';
import type { TimelineParams } from '../types/index.js';

// ========================================================================
// timer/index.d.ts
// ========================================================================
export * from "./timer.js";

// ========================================================================
// timer/timer.d.ts
// ========================================================================
/**
 * Base class used to create Timers, Animations and Timelines
 */
export class Timer extends Clock {
    /**
     * @param {TimerParams} [parameters]
     * @param {Timeline} [parent]
     * @param {Number} [parentPosition]
     */
    constructor(parameters?: TimerParams, parent?: Timeline, parentPosition?: number);
    /** @type {String|Number} */
    id: string | number;
    /** @type {Timeline} */
    parent: Timeline;
    duration: number;
    /** @type {Boolean} */
    backwards: boolean;
    /** @type {Boolean} */
    paused: boolean;
    /** @type {Boolean} */
    began: boolean;
    /** @type {Boolean} */
    completed: boolean;
    /** @type {Callback<this>} */
    onBegin: Callback<this>;
    /** @type {Callback<this>} */
    onBeforeUpdate: Callback<this>;
    /** @type {Callback<this>} */
    onUpdate: Callback<this>;
    /** @type {Callback<this>} */
    onLoop: Callback<this>;
    /** @type {Callback<this>} */
    onPause: Callback<this>;
    /** @type {Callback<this>} */
    onComplete: Callback<this>;
    /** @type {Number} */
    iterationDuration: number;
    /** @type {Number} */
    iterationCount: number;
    /** @type {Boolean|ScrollObserver} */
    _autoplay: boolean | ScrollObserver;
    /** @type {Number} */
    _offset: number;
    /** @type {Number} */
    _delay: number;
    /** @type {Number} */
    _loopDelay: number;
    /** @type {Number} */
    _iterationTime: number;
    /** @type {Number} */
    _currentIteration: number;
    /** @type {Function} */
    _resolve: Function;
    /** @type {Boolean} */
    _running: boolean;
    /** @type {Number} */
    _reversed: number;
    /** @type {Number} */
    _reverse: number;
    /** @type {Number} */
    _cancelled: number;
    /** @type {Boolean} */
    _alternate: boolean;
    /** @type {Renderable} */
    _prev: Renderable;
    /** @type {Renderable} */
    _next: Renderable;
    /** @type {Number} */
    _priority: number;
    set cancelled(cancelled: boolean);
    get cancelled(): boolean;
    set currentTime(time: number);
    get currentTime(): number;
    set iterationCurrentTime(time: number);
    get iterationCurrentTime(): number;
    set progress(progress: number);
    get progress(): number;
    set iterationProgress(progress: number);
    get iterationProgress(): number;
    set currentIteration(iterationCount: number);
    get currentIteration(): number;
    set reversed(reverse: boolean);
    get reversed(): boolean;
    /**
     * @param  {Boolean} [softReset]
     * @return {this}
     */
    reset(softReset?: boolean): this;
    /**
     * @param  {Boolean} internalRender
     * @return {this}
     */
    init(internalRender?: boolean): this;
    /** @return {this} */
    resetTime(): this;
    /** @return {this} */
    pause(): this;
    /** @return {this} */
    resume(): this;
    /** @return {this} */
    restart(): this;
    /**
     * @param  {Number} time
     * @param  {Boolean|Number} [muteCallbacks]
     * @param  {Boolean|Number} [internalRender]
     * @return {this}
     */
    seek(time: number, muteCallbacks?: boolean | number, internalRender?: boolean | number): this;
    /** @return {this} */
    alternate(): this;
    /** @return {this} */
    play(): this;
    /** @return {this} */
    reverse(): this;
    /** @return {this} */
    cancel(): this;
    /**
     * @param  {Number} newDuration
     * @return {this}
     */
    stretch(newDuration: number): this;
    /**
      * Cancels the timer by seeking it back to 0 and reverting the attached scroller if necessary
      * @return {this}
      */
    revert(): this;
    /**
      * Imediatly completes the timer, cancels it and triggers the onComplete callback
      * @param  {Boolean|Number} [muteCallbacks]
      * @return {this}
      */
    complete(muteCallbacks?: boolean | number): this;
    /**
     * @typedef {this & {then: null}} ResolvedTimer
     */
    /**
     * @param  {Callback<ResolvedTimer>} [callback]
     * @return Promise<this>
     */
    then(callback?: Callback<this & {
        then: null;
    }>): Promise<any>;
}
export function createTimer(parameters?: TimerParams): Timer;
import { Clock } from '../core/clock.js';
import type { Timeline } from '../timeline/timeline.js';
import type { Callback } from '../types/index.js';
import type { ScrollObserver } from '../events/scroll.js';
import type { Renderable } from '../types/index.js';
import type { TimerParams } from '../types/index.js';

// ========================================================================
// types/index.d.ts
// ========================================================================
export type DefaultsParams = {
    id?: number | string;
    keyframes?: PercentageKeyframes | DurationKeyframes;
    playbackEase?: EasingParam;
    playbackRate?: number;
    frameRate?: number;
    loop?: number | boolean;
    reversed?: boolean;
    alternate?: boolean;
    persist?: boolean;
    autoplay?: boolean | ScrollObserver;
    duration?: number | FunctionValue;
    delay?: number | FunctionValue;
    loopDelay?: number;
    ease?: EasingParam | FunctionValue;
    composition?: "none" | "replace" | "blend" | compositionTypes;
    modifier?: (v: any) => any;
    onBegin?: Callback<Tickable>;
    onBeforeUpdate?: Callback<Tickable>;
    onUpdate?: Callback<Tickable>;
    onLoop?: Callback<Tickable>;
    onPause?: Callback<Tickable>;
    onComplete?: Callback<Tickable>;
    onRender?: Callback<Renderable>;
};
export type Renderable = JSAnimation | Timeline;
export type Tickable = Timer | Renderable;
export type CallbackArgument = Timer & JSAnimation & Timeline;
export type Revertible = Animatable | Tickable | WAAPIAnimation | Draggable | ScrollObserver | TextSplitter | Scope | AutoLayout;
export type TweakRegister = {
    type: string;
    defaultValue: any;
};
export type StaggerFunction<T> = (target?: Target, index?: number, targets?: TargetsArray, prevTween?: Tween | null, tl?: Timeline) => T;
export type StaggerParams = {
    start?: number | string;
    from?: number | "first" | "center" | "last" | "random" | Array<number>;
    reversed?: boolean;
    grid?: Array<number> | boolean;
    axis?: ("x" | "y" | "z");
    use?: string | {
        method(target: Target, i: number, length: number): number;
    }["method"];
    total?: number;
    ease?: EasingParam;
    modifier?: TweenModifier;
    /**
     * Additive uniform noise on the
     * computed stagger value. Number form gives flat `+/-jitter`; tuple form
     * ramps the magnitude `start -> end` across the from/axis/grid ordering
     * and respects `ease`.
     */
    jitter?: number | [number, number];
    /**
     * Seed for jitter draws and `from: 'random'`
     * shuffling. `false` (default) uses Math.random. `true` seeds with `0`. A
     * number is used directly as the seed.
     */
    seed?: boolean | number;
};
export type DOMTarget = HTMLElement | SVGElement;
export type JSTarget = Record<string, any>;
export type Target = DOMTarget | JSTarget;
export type TargetSelector = Target | NodeList | string;
export type DOMTargetSelector = DOMTarget | NodeList | string;
export type DOMTargetsParam = Array<DOMTargetSelector> | DOMTargetSelector;
export type DOMTargetsArray = Array<DOMTarget>;
export type JSTargetsParam = Array<JSTarget> | JSTarget;
export type JSTargetsArray = Array<JSTarget>;
export type TargetsParam = Array<TargetSelector> | TargetSelector;
export type TargetsArray = Array<Target>;
export type EasingFunction = (time: number) => number;
export type EaseStringParamNames = ("linear" | "none" | "in" | "out" | "inOut" | "inQuad" | "outQuad" | "inOutQuad" | "inCubic" | "outCubic" | "inOutCubic" | "inQuart" | "outQuart" | "inOutQuart" | "inQuint" | "outQuint" | "inOutQuint" | "inSine" | "outSine" | "inOutSine" | "inCirc" | "outCirc" | "inOutCirc" | "inExpo" | "outExpo" | "inOutExpo" | "inBounce" | "outBounce" | "inOutBounce" | "inBack" | "outBack" | "inOutBack" | "inElastic" | "outElastic" | "inOutElastic" | "out(p = 1.675)" | "inOut(p = 1.675)" | "inBack(overshoot = 1.7)" | "outBack(overshoot = 1.7)" | "inOutBack(overshoot = 1.7)" | "inElastic(amplitude = 1, period = .3)" | "outElastic(amplitude = 1, period = .3)" | "inOutElastic(amplitude = 1, period = .3)");
export type WAAPIEaseStringParamNames = ("ease" | "ease-in" | "ease-out" | "ease-in-out" | "linear(0, 0.25, 1)" | "steps" | "steps(6, start)" | "step-start" | "step-end" | "cubic-bezier(0.42, 0, 1, 1)");
export type PowerEasing = (power?: number | string) => EasingFunction;
export type BackEasing = (overshoot?: number | string) => EasingFunction;
export type ElasticEasing = (amplitude?: number | string, period?: number | string) => EasingFunction;
export type EasingFunctionWithParams = PowerEasing | BackEasing | ElasticEasing;
export type EasingParam = (string & {}) | EaseStringParamNames | EasingFunction | Spring | TweakRegister;
export type WAAPIEasingParam = (string & {}) | EaseStringParamNames | WAAPIEaseStringParamNames | EasingFunction | Spring | TweakRegister;
export type SpringParams = {
    /**
     * - Mass, default 1
     */
    mass?: number;
    /**
     * - Stiffness, default 100
     */
    stiffness?: number;
    /**
     * - Damping, default 10
     */
    damping?: number;
    /**
     * - Initial velocity, default 0
     */
    velocity?: number;
    /**
     * - Initial bounce, default 0
     */
    bounce?: number;
    /**
     * - The perceived duration, default 0
     */
    duration?: number;
    /**
     * - Callback function called when the spring currentTime hits the perceived duration
     */
    onComplete?: Callback<JSAnimation>;
};
export type Callback<T> = {
    method(self: T): any;
}["method"];
export type TickableCallbacks<T extends unknown> = {
    onBegin?: Callback<T>;
    onBeforeUpdate?: Callback<T>;
    onUpdate?: Callback<T>;
    onLoop?: Callback<T>;
    onPause?: Callback<T>;
    onComplete?: Callback<T>;
};
export type RenderableCallbacks<T extends unknown> = {
    onRender?: Callback<T>;
};
export type TimerOptions = {
    id?: number | string;
    duration?: TweenParamValue;
    delay?: TweenParamValue;
    loopDelay?: number;
    reversed?: boolean;
    alternate?: boolean;
    loop?: boolean | number;
    autoplay?: boolean | ScrollObserver;
    frameRate?: number;
    playbackRate?: number;
    priority?: number;
};
export type TimerParams = TimerOptions & TickableCallbacks<Timer>;
export type FunctionValueReturn = number | string | TweenKeyValue | EasingParam | Array<number | string | TweenKeyValue>;
export type FunctionValue<T = FunctionValueReturn> = (target?: Target, index?: number, targets?: TargetsArray, prevTween?: Tween | null) => T;
export type TweenModifier = (value: number) => number | string;
export type ColorArray = [number, number, number, number];
export type Tween = {
    id: number;
    parent: JSAnimation;
    property: string;
    target: Target;
    _value: string | number | any;
    _toFunc: Function | null;
    _fromFunc: Function | null;
    _ease: EasingFunction;
    _fromNumbers: Array<number>;
    _toNumbers: Array<number>;
    _strings: Array<string>;
    _fromNumber: number;
    _toNumber: number;
    _numbers: Array<number>;
    _number: number;
    _unit: string;
    _modifier: TweenModifier;
    _currentTime: number;
    _delay: number;
    _updateDuration: number;
    _startTime: number;
    _changeDuration: number;
    _absoluteStartTime: number;
    _absoluteUpdateStartTime: number;
    _absoluteEndTime: number;
    _hasFromValue: number;
    _tweenType: tweenTypes;
    _setter: ((target: any, value: number, tween: Tween) => void) | null;
    _valueType: valueTypes;
    _composition: number;
    _isOverlapped: number;
    _isOverridden: number;
    _renderTransforms: number;
    _inlineValue: string;
    _prevRep: Tween;
    _nextRep: Tween;
    _prevAdd: Tween;
    _nextAdd: Tween;
    _prev: Tween;
    _next: Tween;
};
export type TweenDecomposedValue = {
    /**
     * - Type
     */
    t: number;
    /**
     * - Single number value
     */
    n: number;
    /**
     * - Value unit
     */
    u: string;
    /**
     * - Value operator
     */
    o: string;
    /**
     * - Array of Numbers (complex / color value type)
     */
    d: Array<number>;
    /**
     * - Strings (complex value type)
     */
    s: Array<string>;
};
export type TweenPropertySiblings = {
    _head: null | Tween;
    _tail: null | Tween;
};
export type TweenLookups = Record<string, TweenPropertySiblings>;
export type TweenReplaceLookups = WeakMap<Target, TweenLookups>;
export type TweenAdditiveLookups = Map<Target, TweenLookups>;
export type TweenParamValue = number | string | FunctionValue | EasingParam | TweakRegister;
export type TweenPropValue = TweenParamValue | [TweenParamValue, TweenParamValue];
export type TweenComposition = (string & {}) | "none" | "replace" | "blend" | compositionTypes;
export type TweenParamsOptions = {
    duration?: TweenParamValue;
    delay?: TweenParamValue;
    ease?: EasingParam | FunctionValue;
    modifier?: TweenModifier;
    composition?: TweenComposition;
};
export type TweenValues = {
    from?: TweenParamValue;
    to?: TweenPropValue;
    fromTo?: TweenPropValue;
};
export type TweenKeyValue = TweenParamsOptions & TweenValues;
export type ArraySyntaxValue = Array<TweenKeyValue | TweenPropValue>;
export type TweenOptions = TweenParamValue | ArraySyntaxValue | TweenKeyValue;
export type TweenObjectValue = Partial<{
    to: TweenParamValue | Array<TweenParamValue>;
    from: TweenParamValue | Array<TweenParamValue>;
    fromTo: TweenParamValue | Array<TweenParamValue>;
}>;
export type PercentageKeyframeOptions = {
    ease?: EasingParam;
};
export type PercentageKeyframeParams = Record<string, TweenParamValue>;
export type PercentageKeyframes = Record<string, PercentageKeyframeParams & PercentageKeyframeOptions>;
export type DurationKeyframes = Array<Record<string, TweenOptions | TweenModifier | boolean> & TweenParamsOptions>;
export type AnimationOptions = {
    keyframes?: PercentageKeyframes | DurationKeyframes;
    playbackEase?: EasingParam;
};
export type AnimationParams = Record<string, TweenOptions | Callback<JSAnimation> | TweenModifier | boolean | PercentageKeyframes | DurationKeyframes | ScrollObserver> & TimerOptions & AnimationOptions & TweenParamsOptions & TickableCallbacks<JSAnimation> & RenderableCallbacks<JSAnimation>;
/**
 * Accepts:<br>
 * - `Number` - Absolute position in milliseconds (e.g., `500` places element at exactly 500ms)<br>
 * - `'+=Number'` - Addition: Position element X ms after the last element (e.g., `'+=100'`)<br>
 * - `'-=Number'` - Subtraction: Position element X ms before the last element's end (e.g., `'-=100'`)<br>
 * - `'*=Number'` - Multiplier: Position element at a fraction of the total duration (e.g., `'*=.5'` for halfway)<br>
 * - `'<'` - Previous end: Position element at the end position of the previous element<br>
 * - `'<<'` - Previous start: Position element at the start position of the previous element<br>
 * - `'<<+=Number'` - Combined: Position element relative to previous element's start (e.g., `'<<+=250'`)<br>
 * - `'label'` - Label: Position element at a named label position (e.g., `'My Label'`)
 */
export type TimelinePosition = number | `+=${number}` | `-=${number}` | `*=${number}` | "<" | "<<" | `<<+=${number}` | `<<-=${number}` | string;
/**
 * Accepts:<br>
 * - `Number` - Absolute position in milliseconds (e.g., `500` places animation at exactly 500ms)<br>
 * - `'+=Number'` - Addition: Position animation X ms after the last animation (e.g., `'+=100'`)<br>
 * - `'-=Number'` - Subtraction: Position animation X ms before the last animation's end (e.g., `'-=100'`)<br>
 * - `'*=Number'` - Multiplier: Position animation at a fraction of the total duration (e.g., `'*=.5'` for halfway)<br>
 * - `'<'` - Previous end: Position animation at the end position of the previous animation<br>
 * - `'<<'` - Previous start: Position animation at the start position of the previous animation<br>
 * - `'<<+=Number'` - Combined: Position animation relative to previous animation's start (e.g., `'<<+=250'`)<br>
 * - `'label'` - Label: Position animation at a named label position (e.g., `'My Label'`)<br>
 * - `stagger(String|Nummber)` - Stagger multi-elements animation positions (e.g., 10, 20, 30...)
 */
export type TimelineAnimationPosition = TimelinePosition | StaggerFunction<number | string> | TweakRegister;
export type TimelineOptions = {
    defaults?: DefaultsParams;
    playbackEase?: EasingParam;
    composition?: boolean;
};
export type TimelineParams = TimerOptions & TimelineOptions & TickableCallbacks<Timeline> & RenderableCallbacks<Timeline>;
export type WAAPITweenValue = string | number | Array<string> | Array<number>;
export type WAAPIFunctionValue = (target: DOMTarget, index: number, targets: DOMTargetsArray) => WAAPITweenValue | WAAPIEasingParam;
export type WAAPIKeyframeValue = WAAPITweenValue | WAAPIFunctionValue | Array<string | number | WAAPIFunctionValue>;
export type WAAPITweenOptions = {
    to?: WAAPIKeyframeValue;
    from?: WAAPIKeyframeValue;
    duration?: number | WAAPIFunctionValue;
    delay?: number | WAAPIFunctionValue;
    ease?: WAAPIEasingParam;
    composition?: CompositeOperation;
};
export type WAAPIAnimationOptions = {
    loop?: number | boolean;
    Reversed?: boolean;
    Alternate?: boolean;
    autoplay?: boolean | ScrollObserver;
    playbackRate?: number;
    duration?: number | WAAPIFunctionValue;
    delay?: number | WAAPIFunctionValue;
    ease?: WAAPIEasingParam | WAAPIFunctionValue;
    composition?: CompositeOperation;
    persist?: boolean;
    onComplete?: Callback<WAAPIAnimation>;
};
export type WAAPIAnimationParams = Record<string, WAAPIKeyframeValue | WAAPIAnimationOptions | boolean | ScrollObserver | Callback<WAAPIAnimation> | WAAPIEasingParam | WAAPITweenOptions> & WAAPIAnimationOptions;
export type AnimatablePropertySetter = (to: number | Array<number>, duration?: number, ease?: EasingParam) => AnimatableObject;
export type AnimatablePropertyGetter = () => number | Array<number>;
export type AnimatableProperty = AnimatablePropertySetter & AnimatablePropertyGetter;
export type AnimatableObject = Animatable & Record<string, AnimatableProperty>;
export type AnimatablePropertyParamsOptions = {
    unit?: string;
    duration?: TweenParamValue;
    ease?: EasingParam;
    modifier?: TweenModifier;
    composition?: TweenComposition;
};
export type AnimatableParams = Record<string, TweenParamValue | EasingParam | TweenModifier | TweenComposition | AnimatablePropertyParamsOptions | Callback<JSAnimation>> & AnimatablePropertyParamsOptions & TickableCallbacks<JSAnimation> & RenderableCallbacks<JSAnimation>;
export type ReactRef = {
    current?: HTMLElement | SVGElement | null;
};
export type AngularRef = {
    nativeElement?: HTMLElement | SVGElement;
};
export type ScopeParams = {
    root?: DOMTargetSelector | ReactRef | AngularRef;
    defaults?: DefaultsParams;
    mediaQueries?: Record<string, string>;
};
export type ScopedCallback<T> = (scope: Scope) => T;
export type ScopeCleanupCallback = (scope?: Scope) => any;
export type ScopeConstructorCallback = (scope?: Scope) => ScopeCleanupCallback | void;
export type ScopeMethod = (...args: any[]) => any;
export type ScrollThresholdValue = string | number;
export type ScrollThresholdParam = {
    target?: ScrollThresholdValue;
    container?: ScrollThresholdValue;
};
export type ScrollObserverAxisCallback = (self: ScrollObserver) => "x" | "y";
export type ScrollThresholdCallback = (self: ScrollObserver) => ScrollThresholdValue | ScrollThresholdParam;
export type ScrollObserverParams = {
    id?: number | string;
    sync?: boolean | number | string | EasingParam;
    container?: TargetsParam;
    target?: TargetsParam;
    axis?: "x" | "y" | ScrollObserverAxisCallback | ((observer: ScrollObserver) => "x" | "y" | ScrollObserverAxisCallback);
    enter?: ScrollThresholdValue | ScrollThresholdParam | ScrollThresholdCallback | ((observer: ScrollObserver) => ScrollThresholdValue | ScrollThresholdParam | ScrollThresholdCallback);
    leave?: ScrollThresholdValue | ScrollThresholdParam | ScrollThresholdCallback | ((observer: ScrollObserver) => ScrollThresholdValue | ScrollThresholdParam | ScrollThresholdCallback);
    repeat?: boolean | ((observer: ScrollObserver) => boolean);
    debug?: boolean;
    onEnter?: Callback<ScrollObserver>;
    onLeave?: Callback<ScrollObserver>;
    onEnterForward?: Callback<ScrollObserver>;
    onLeaveForward?: Callback<ScrollObserver>;
    onEnterBackward?: Callback<ScrollObserver>;
    onLeaveBackward?: Callback<ScrollObserver>;
    onUpdate?: Callback<ScrollObserver>;
    onResize?: Callback<ScrollObserver>;
    onSyncComplete?: Callback<ScrollObserver>;
};
export type DraggableAxisParam = {
    mapTo?: string;
    modifier?: TweenModifier;
    composition?: TweenComposition;
    snap?: number | Array<number> | ((draggable: Draggable) => number | Array<number>);
};
export type DraggableCursorParams = {
    onHover?: string;
    onGrab?: string;
};
export type DraggableDragThresholdParams = {
    mouse?: number;
    touch?: number;
};
export type DraggableParams = {
    trigger?: DOMTargetSelector;
    container?: DOMTargetSelector | Array<number> | ((draggable: Draggable) => DOMTargetSelector | Array<number>);
    x?: boolean | DraggableAxisParam;
    y?: boolean | DraggableAxisParam;
    modifier?: TweenModifier;
    snap?: number | Array<number> | ((draggable: Draggable) => number | Array<number>);
    containerPadding?: number | Array<number> | ((draggable: Draggable) => number | Array<number>);
    containerFriction?: number | ((draggable: Draggable) => number);
    releaseContainerFriction?: number | ((draggable: Draggable) => number);
    dragSpeed?: number | ((draggable: Draggable) => number);
    dragThreshold?: number | DraggableDragThresholdParams | ((draggable: Draggable) => number | DraggableDragThresholdParams);
    scrollSpeed?: number | ((draggable: Draggable) => number);
    scrollThreshold?: number | ((draggable: Draggable) => number);
    minVelocity?: number | ((draggable: Draggable) => number);
    maxVelocity?: number | ((draggable: Draggable) => number);
    velocityMultiplier?: number | ((draggable: Draggable) => number);
    releaseMass?: number;
    releaseStiffness?: number;
    releaseDamping?: number;
    releaseEase?: EasingParam;
    cursor?: boolean | DraggableCursorParams | ((draggable: Draggable) => boolean | DraggableCursorParams);
    onGrab?: Callback<Draggable>;
    onDrag?: Callback<Draggable>;
    onRelease?: Callback<Draggable>;
    onUpdate?: Callback<Draggable>;
    onSettle?: Callback<Draggable>;
    onSnap?: Callback<Draggable>;
    onResize?: Callback<Draggable>;
    onAfterResize?: Callback<Draggable>;
};
export type SplitTemplateParams = {
    class?: false | string;
    wrap?: boolean | "hidden" | "clip" | "visible" | "scroll" | "auto";
    clone?: boolean | "top" | "right" | "bottom" | "left" | "center";
};
export type SplitValue = boolean | string;
export type SplitFunctionValue = (value?: Node | HTMLElement) => any;
export type TextSplitterParams = {
    lines?: SplitValue | SplitTemplateParams | SplitFunctionValue;
    words?: SplitValue | SplitTemplateParams | SplitFunctionValue;
    chars?: SplitValue | SplitTemplateParams | SplitFunctionValue;
    accessible?: boolean;
    includeSpaces?: boolean;
    debug?: boolean;
};
export type ScrambleTextParams = {
    /**
     * - the text to transition to, otherwise uses the original text
     */
    text?: string | ((arg0: Target, arg1: number, arg2: TargetsArray) => string);
    /**
     * - the characters used for scramble; named sets: 'lowercase', 'uppercase', 'numbers', 'symbols', 'braille', 'blocks', 'shades'; range syntax: 'A-Z', 'a-z0-9'; defaults to 'a-zA-Z0-9!%#_'
     */
    chars?: string | ((arg0: Target, arg1: number, arg2: TargetsArray) => string);
    /**
     * - the easing applied to the scramble animation
     */
    ease?: EasingParam;
    /**
     * - where the reveal wave starts from, 'auto' (default) uses 'left' when text grows and 'right' when it shrinks
     */
    from?: number | "left" | "center" | "right" | "random" | "auto";
    /**
     * - reverses the reveal order, so 'center' reveals from edges inward instead of center outward
     */
    reversed?: boolean;
    /**
     * - characters displayed at the leading edge of the reveal wave; true uses '_', a number is a char code, a string is used directly
     */
    cursor?: boolean | number | string;
    /**
     * - adds random timing offsets to each character's start and end, creating a more organic reveal
     */
    perturbation?: number;
    /**
     * - a seed for the random number generator to produce reproducible scramble sequences
     */
    seed?: number;
    /**
     * - controls the starting appearance: false shows original text, true scrambles it (default), '' starts from blank, ' ' replaces characters with spaces, a custom string (supports range syntax like 'A-Z') uses its characters as scramble set
     */
    override?: boolean | string;
    /**
     * - characters per second entering the active zone; higher values make the reveal wave move faster (default: 60)
     */
    revealRate?: number;
    /**
     * - time in ms each character spends scrambling before settling into its final glyph (default: 300)
     */
    settleDuration?: number;
    /**
     * - how many times per second scramble characters cycle in the active zone (default: 30)
     */
    settleRate?: number;
    /**
     * - if set to a value greater than 0, overrides the computed duration from interval and settle; if unset or 0, duration is calculated automatically from text length and timing parameters
     */
    duration?: number | ((arg0: Target, arg1: number, arg2: TargetsArray) => number);
    /**
     * - delay in ms before the reveal wave starts within the scramble animation
     */
    revealDelay?: number | ((arg0: Target, arg1: number, arg2: TargetsArray) => number);
    /**
     * - delay in ms before the entire scramble animation starts
     */
    delay?: number | ((arg0: Target, arg1: number, arg2: TargetsArray) => number);
    /**
     * - callback fired each time a character changes during scramble; receives the current scrambled text and the eased progress (0-1)
     */
    onChange?: (arg0: string, arg1: number) => void;
};
export type DrawableSVGGeometry = SVGGeometryElement & {
    setAttribute(name: "draw", value: `${number} ${number}`): void;
    draw: `${number} ${number}`;
};
import type { ScrollObserver } from '../events/scroll.js';
import type { compositionTypes } from '../core/consts.js';
import type { JSAnimation } from '../animation/animation.js';
import type { Timeline } from '../timeline/timeline.js';
import type { Timer } from '../timer/timer.js';
import type { Animatable } from '../animatable/animatable.js';
import type { WAAPIAnimation } from '../waapi/waapi.js';
import type { Draggable } from '../draggable/draggable.js';
import type { TextSplitter } from '../text/split.js';
import type { Scope } from '../scope/scope.js';
import type { AutoLayout } from '../layout/layout.js';
import type { Spring } from '../easings/spring/index.js';
import type { tweenTypes } from '../core/consts.js';
import type { valueTypes } from '../core/consts.js';

// ========================================================================
// utils/chainable.d.ts
// ========================================================================
/**
 * @typedef {Object} ChainablesMap
 * @property {ChainedClamp} clamp
 * @property {ChainedRound} round
 * @property {ChainedSnap} snap
 * @property {ChainedWrap} wrap
 * @property {ChainedLerp} lerp
 * @property {ChainedDamp} damp
 * @property {ChainedMapRange} mapRange
 * @property {ChainedRoundPad} roundPad
 * @property {ChainedPadStart} padStart
 * @property {ChainedPadEnd} padEnd
 * @property {ChainedDegToRad} degToRad
 * @property {ChainedRadToDeg} radToDeg
 */
/**
 * @callback ChainedUtilsResult
 * @param {Number} value - The value to process through the chained operations
 * @return {Number} The processed result
 */
/**
 * @typedef {ChainablesMap & ChainedUtilsResult} ChainableUtil
 */
/**
 * @callback ChainedRoundPad
 * @param {Number} decimalLength - Number of decimal places
 * @return {ChainableUtil}
 */
export const roundPad: typeof numberUtils.roundPad & ChainedRoundPad;
/**
 * @callback ChainedPadStart
 * @param {Number} totalLength - Target length
 * @param {String} padString - String to pad with
 * @return {ChainableUtil}
 */
export const padStart: typeof numberUtils.padStart & ChainedPadStart;
/**
 * @callback ChainedPadEnd
 * @param {Number} totalLength - Target length
 * @param {String} padString - String to pad with
 * @return {ChainableUtil}
 */
export const padEnd: typeof numberUtils.padEnd & ChainedPadEnd;
/**
 * @callback ChainedWrap
 * @param {Number} min - Minimum boundary
 * @param {Number} max - Maximum boundary
 * @return {ChainableUtil}
 */
export const wrap: typeof numberUtils.wrap & ChainedWrap;
/**
 * @callback ChainedMapRange
 * @param {Number} inLow - Input range minimum
 * @param {Number} inHigh - Input range maximum
 * @param {Number} outLow - Output range minimum
 * @param {Number} outHigh - Output range maximum
 * @return {ChainableUtil}
 */
export const mapRange: typeof numberUtils.mapRange & ChainedMapRange;
/**
 * @callback ChainedDegToRad
 * @return {ChainableUtil}
 */
export const degToRad: typeof numberUtils.degToRad & ChainedDegToRad;
/**
 * @callback ChainedRadToDeg
 * @return {ChainableUtil}
 */
export const radToDeg: typeof numberUtils.radToDeg & ChainedRadToDeg;
/**
 * @callback ChainedSnap
 * @param {Number|Array<Number>} increment - Step size or array of snap points
 * @return {ChainableUtil}
 */
export const snap: typeof numberUtils.snap & ChainedSnap;
/**
 * @callback ChainedClamp
 * @param {Number} min - Minimum boundary
 * @param {Number} max - Maximum boundary
 * @return {ChainableUtil}
 */
export const clamp: typeof numberUtils.clamp & ChainedClamp;
/**
 * @callback ChainedRound
 * @param {Number} decimalLength - Number of decimal places
 * @return {ChainableUtil}
 */
export const round: typeof numberUtils.round & ChainedRound;
/**
 * @callback ChainedLerp
 * @param {Number} start - Starting value
 * @param {Number} end - Ending value
 * @return {ChainableUtil}
 */
export const lerp: typeof numberUtils.lerp & ChainedLerp;
/**
 * @callback ChainedDamp
 * @param {Number} start - Starting value
 * @param {Number} end - Target value
 * @param {Number} deltaTime - Delta time in ms
 * @return {ChainableUtil}
 */
export const damp: typeof numberUtils.damp & ChainedDamp;
export type UtilityFunction = (...args: any[]) => number | string;
export type ChainablesMap = {
    clamp: ChainedClamp;
    round: ChainedRound;
    snap: ChainedSnap;
    wrap: ChainedWrap;
    lerp: ChainedLerp;
    damp: ChainedDamp;
    mapRange: ChainedMapRange;
    roundPad: ChainedRoundPad;
    padStart: ChainedPadStart;
    padEnd: ChainedPadEnd;
    degToRad: ChainedDegToRad;
    radToDeg: ChainedRadToDeg;
};
export type ChainedUtilsResult = (value: number) => number;
export type ChainableUtil = ChainablesMap & ChainedUtilsResult;
export type ChainedRoundPad = (decimalLength: number) => ChainableUtil;
export type ChainedPadStart = (totalLength: number, padString: string) => ChainableUtil;
export type ChainedPadEnd = (totalLength: number, padString: string) => ChainableUtil;
export type ChainedWrap = (min: number, max: number) => ChainableUtil;
export type ChainedMapRange = (inLow: number, inHigh: number, outLow: number, outHigh: number) => ChainableUtil;
export type ChainedDegToRad = () => ChainableUtil;
export type ChainedRadToDeg = () => ChainableUtil;
export type ChainedSnap = (increment: number | Array<number>) => ChainableUtil;
export type ChainedClamp = (min: number, max: number) => ChainableUtil;
export type ChainedRound = (decimalLength: number) => ChainableUtil;
export type ChainedLerp = (start: number, end: number) => ChainableUtil;
export type ChainedDamp = (start: number, end: number, deltaTime: number) => ChainableUtil;
declare const numberUtils: typeof numberImports;
import * as numberImports from './number.js';
export {};

// ========================================================================
// utils/index.d.ts
// ========================================================================
export * from "./chainable.js";
export * from "./random.js";
export * from "./time.js";
export * from "./target.js";
export * from "./stagger.js";
export { forEachChildren, addChild, removeChild } from "../core/helpers.js";

// ========================================================================
// utils/number.d.ts
// ========================================================================
export function roundPad(v: number | string, decimalLength: number): string;
export function padStart(v: number, totalLength: number, padString: string): string;
export function padEnd(v: number, totalLength: number, padString: string): string;
export function wrap(v: number, min: number, max: number): number;
export function mapRange(value: number, inLow: number, inHigh: number, outLow: number, outHigh: number): number;
export function degToRad(degrees: number): number;
export function radToDeg(radians: number): number;
export function damp(start: number, end: number, deltaTime: number, factor: number): number;
export { snap, clamp, round, lerp } from "../core/helpers.js";

// ========================================================================
// utils/random.d.ts
// ========================================================================
/**
 * Generate a random number between optional min and max (inclusive) and decimal precision
 *
 * @callback RandomNumberGenerator
 * @param    {Number} [min=0] - The minimum value (inclusive)
 * @param    {Number} [max=1] - The maximum value (inclusive)
 * @param    {Number} [decimalLength=0] - Number of decimal places to round to
 * @return   {Number} A random number between min and max
 */
/**
 * Generates a random number between min and max (inclusive) with optional decimal precision
 *
 * @type {RandomNumberGenerator}
 */
export const random: RandomNumberGenerator;
export function createSeededRandom(seed?: number, seededMin?: number, seededMax?: number, seededDecimalLength?: number): RandomNumberGenerator;
export function randomPick<T>(items: string | Array<T>): string | T;
export function shuffle(items: any[], rnd?: RandomNumberGenerator): any[];
/**
 * Generate a random number between optional min and max (inclusive) and decimal precision
 */
export type RandomNumberGenerator = (min?: number, max?: number, decimalLength?: number) => number;

// ========================================================================
// utils/stagger.d.ts
// ========================================================================
/**
 * @overload
 * @param {Number} val
 * @param {StaggerParams} [params]
 * @return {StaggerFunction<Number>}
 */
export function stagger(val: number, params?: StaggerParams): StaggerFunction<number>;
/**
 * @overload
 * @param {String} val
 * @param {StaggerParams} [params]
 * @return {StaggerFunction<String>}
 */
export function stagger(val: string, params?: StaggerParams): StaggerFunction<string>;
/**
 * @overload
 * @param {[Number, Number]} val
 * @param {StaggerParams} [params]
 * @return {StaggerFunction<Number>}
 */
export function stagger(val: [number, number], params?: StaggerParams): StaggerFunction<number>;
/**
 * @overload
 * @param {[String, String]} val
 * @param {StaggerParams} [params]
 * @return {StaggerFunction<String>}
 */
export function stagger(val: [string, string], params?: StaggerParams): StaggerFunction<string>;
import type { StaggerParams } from '../types/index.js';
import type { StaggerFunction } from '../types/index.js';

// ========================================================================
// utils/target.d.ts
// ========================================================================
/**
 * @overload
 * @param  {DOMTargetSelector} targetSelector
 * @param  {String} propName
 * @return {String}
 *
 * @overload
 * @param  {JSTargetsParam} targetSelector
 * @param  {String} propName
 * @return {Number|String}
 *
 * @overload
 * @param  {DOMTargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String} unit
 * @return {String}
 *
 * @overload
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {Boolean} unit
 * @return {Number}
 *
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String|Boolean} [unit]
 */
export function get(targetSelector: DOMTargetSelector, propName: string): string;
/**
 * @overload
 * @param  {DOMTargetSelector} targetSelector
 * @param  {String} propName
 * @return {String}
 *
 * @overload
 * @param  {JSTargetsParam} targetSelector
 * @param  {String} propName
 * @return {Number|String}
 *
 * @overload
 * @param  {DOMTargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String} unit
 * @return {String}
 *
 * @overload
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {Boolean} unit
 * @return {Number}
 *
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String|Boolean} [unit]
 */
export function get(targetSelector: JSTargetsParam, propName: string): number | string;
/**
 * @overload
 * @param  {DOMTargetSelector} targetSelector
 * @param  {String} propName
 * @return {String}
 *
 * @overload
 * @param  {JSTargetsParam} targetSelector
 * @param  {String} propName
 * @return {Number|String}
 *
 * @overload
 * @param  {DOMTargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String} unit
 * @return {String}
 *
 * @overload
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {Boolean} unit
 * @return {Number}
 *
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String|Boolean} [unit]
 */
export function get(targetSelector: DOMTargetsParam, propName: string, unit: string): string;
/**
 * @overload
 * @param  {DOMTargetSelector} targetSelector
 * @param  {String} propName
 * @return {String}
 *
 * @overload
 * @param  {JSTargetsParam} targetSelector
 * @param  {String} propName
 * @return {Number|String}
 *
 * @overload
 * @param  {DOMTargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String} unit
 * @return {String}
 *
 * @overload
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {Boolean} unit
 * @return {Number}
 *
 * @param  {TargetsParam} targetSelector
 * @param  {String} propName
 * @param  {String|Boolean} [unit]
 */
export function get(targetSelector: TargetsParam, propName: string, unit: boolean): number;
export { registerTargets as $ };
export function set(targets: TargetsParam, parameters: AnimationParams): JSAnimation;
export function remove(targets: TargetsParam, renderable?: Renderable | WAAPIAnimation, propertyName?: string): TargetsArray;
export { cleanInlineStyles } from "../core/styles.js";
import type { DOMTargetSelector } from '../types/index.js';
import type { JSTargetsParam } from '../types/index.js';
import type { DOMTargetsParam } from '../types/index.js';
import type { TargetsParam } from '../types/index.js';
import { registerTargets } from '../core/targets.js';
import type { AnimationParams } from '../types/index.js';
import { JSAnimation } from '../animation/animation.js';
import type { Renderable } from '../types/index.js';
import type { WAAPIAnimation } from '../waapi/waapi.js';
import type { TargetsArray } from '../types/index.js';

// ========================================================================
// utils/time.d.ts
// ========================================================================
export function sync(callback?: Callback<Timer>): Timer;
export function keepTime<T extends Tickable | ((...args: any[]) => void) | void>(constructor: (...args: any[]) => T): (...args: any[]) => T extends void ? () => void : T;
import { Timer } from '../timer/timer.js';
import type { Callback } from '../types/index.js';
import type { Tickable } from '../types/index.js';

// ========================================================================
// waapi/composition.d.ts
// ========================================================================
export function removeWAAPIAnimation($el: DOMTarget, property?: string, parent?: WAAPIAnimation): globalThis.Animation;
export function addWAAPIAnimation(parent: WAAPIAnimation, $el: DOMTarget, property: string, keyframes: PropertyIndexedKeyframes, params: KeyframeAnimationOptions): Animation;
import type { DOMTarget } from '../types/index.js';
import type { WAAPIAnimation } from '../waapi/waapi.js';

// ========================================================================
// waapi/index.d.ts
// ========================================================================
export * from "./waapi.js";

// ========================================================================
// waapi/waapi.d.ts
// ========================================================================
export class WAAPIAnimation {
    /**
     * @param {DOMTargetsParam} targets
     * @param {WAAPIAnimationParams} params
     */
    constructor(targets: DOMTargetsParam, params: WAAPIAnimationParams);
    /** @type {DOMTargetsArray}] */
    targets: DOMTargetsArray;
    /** @type {Array<globalThis.Animation>}] */
    animations: Array<globalThis.Animation>;
    /** @type {globalThis.Animation}] */
    controlAnimation: globalThis.Animation;
    /** @type {Callback<this>} */
    onComplete: Callback<this>;
    /** @type {Number} */
    duration: number;
    /** @type {Boolean} */
    muteCallbacks: boolean;
    /** @type {Boolean} */
    completed: boolean;
    /** @type {Boolean} */
    paused: boolean;
    /** @type {Boolean} */
    reversed: boolean;
    /** @type {Boolean} */
    persist: boolean;
    /** @type {Boolean|ScrollObserver} */
    autoplay: boolean | ScrollObserver;
    /** @type {Number} */
    _speed: number;
    /** @type {Function} */
    _resolve: Function;
    /** @type {Number} */
    _completed: number;
    /** @type {Array.<Object>} */
    _inlineStyles: Array<any>;
    /**
     * @callback forEachCallback
     * @param {globalThis.Animation} animation
     */
    /**
     * @param  {forEachCallback|String} callback
     * @return {this}
     */
    forEach(callback: ((animation: globalThis.Animation) => any) | string): this;
    set speed(speed: number);
    get speed(): number;
    set currentTime(time: number);
    get currentTime(): number;
    set progress(progress: number);
    get progress(): number;
    resume(): this;
    pause(): this;
    alternate(): this;
    play(): this;
    reverse(): this;
    /**
     * @param {Number} time
     * @param {Boolean} muteCallbacks
     */
    seek(time: number, muteCallbacks?: boolean): this;
    restart(): this;
    commitStyles(): this;
    complete(): this;
    cancel(): this;
    revert(): this;
    /**
     * @typedef {this & {then: null}} ResolvedWAAPIAnimation
     */
    /**
     * @param  {Callback<ResolvedWAAPIAnimation>} [callback]
     * @return Promise<this>
     */
    then(callback?: Callback<this & {
        then: null;
    }>): Promise<any>;
}
export namespace waapi {
    export function animate(targets: DOMTargetsParam, params: WAAPIAnimationParams): WAAPIAnimation;
    export { easingToLinear as convertEase };
}
import type { DOMTargetsArray } from '../types/index.js';
import type { Callback } from '../types/index.js';
import type { ScrollObserver } from '../events/scroll.js';
import type { DOMTargetsParam } from '../types/index.js';
import type { WAAPIAnimationParams } from '../types/index.js';
/**
 * @import {
 *   Callback,
 *   EasingFunction,
 *   EasingParam,
 *   DOMTarget,
 *   DOMTargetsParam,
 *   DOMTargetsArray,
 *   WAAPIAnimationParams,
 *   WAAPITweenOptions,
 *   WAAPIKeyframeValue,
 *   WAAPITweenValue
 * } from '../types/index.js'
*/
/**
 * @import {
 *   Spring,
 * } from '../easings/spring/index.js'
*/
/**
 * @import {
 *   ScrollObserver,
 * } from '../events/scroll.js'
*/
/**
 * Converts an easing function into a valid CSS linear() timing function string
 * @param {EasingFunction} fn
 * @param {number} [samples=100]
 * @returns {string} CSS linear() timing function
 */
declare function easingToLinear(fn: EasingFunction, samples?: number): string;
import type { EasingFunction } from '../types/index.js';
export {};
