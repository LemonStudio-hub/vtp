import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
	workerStore,
	events,
	workerState,
	progress,
	addEvent,
	resetWorkerState
} from '../src/stores/worker';

describe('workerStore', () => {
	it('initializes with null', () => {
		expect(get(workerStore)).toBeNull();
	});
});

describe('events', () => {
	it('initializes with empty array', () => {
		expect(get(events)).toEqual([]);
	});

	it('stores added events', () => {
		events.set([]);
		addEvent({ type: 'info', message: 'test event' });
		const evts = get(events);
		expect(evts).toHaveLength(1);
		expect(evts[0].type).toBe('info');
		expect(evts[0].message).toBe('test event');
		expect(evts[0].timestamp).toBeTypeOf('number');
	});

	it('prepends new events', () => {
		events.set([]);
		addEvent({ type: 'info', message: 'first' });
		addEvent({ type: 'checkpoint', message: 'second' });

		const evts = get(events);
		expect(evts).toHaveLength(2);
		expect(evts[0].message).toBe('second');
		expect(evts[1].message).toBe('first');
	});

	it('limits events to 50', () => {
		events.set([]);
		for (let i = 0; i < 55; i++) {
			addEvent({ type: 'info', message: `event ${i}` });
		}
		expect(get(events)).toHaveLength(50);
	});

	it('preserves event types correctly', () => {
		events.set([]);
		addEvent({ type: 'info', message: 'info' });
		addEvent({ type: 'checkpoint', message: 'checkpoint' });
		addEvent({ type: 'winner', message: 'winner' });
		addEvent({ type: 'error', message: 'error' });

		const evts = get(events);
		expect(evts[3].type).toBe('info');
		expect(evts[2].type).toBe('checkpoint');
		expect(evts[1].type).toBe('winner');
		expect(evts[0].type).toBe('error');
	});
});

describe('workerState', () => {
	beforeEach(() => {
		resetWorkerState();
	});

	it('initializes with correct defaults', () => {
		const state = get(workerState);
		expect(state.isRunning).toBe(false);
		expect(state.isPaused).toBe(false);
		expect(state.currentStep).toBe(0);
		expect(state.totalSteps).toBe(0);
		expect(state.speed).toBe(0);
		expect(state.uptime).toBe(0);
		expect(state.winnerCount).toBe(0);
		expect(state.luckPercentage).toBe(100);
		expect(state.publicKey).toBeNull();
		expect(state.nodeId).toBe('---');
	});

	it('can be updated', () => {
		workerState.update((s) => ({
			...s,
			isRunning: true,
			currentStep: 1000,
			speed: 5000
		}));

		const state = get(workerState);
		expect(state.isRunning).toBe(true);
		expect(state.currentStep).toBe(1000);
		expect(state.speed).toBe(5000);
	});
});

describe('progress', () => {
	beforeEach(() => {
		resetWorkerState();
	});

	it('returns 0 when totalSteps is 0', () => {
		expect(get(progress)).toBe(0);
	});

	it('calculates correct progress', () => {
		workerState.update((s) => ({
			...s,
			currentStep: 500,
			totalSteps: 1000
		}));
		expect(get(progress)).toBe(0.5);
	});

	it('returns 1 when complete', () => {
		workerState.update((s) => ({
			...s,
			currentStep: 1000,
			totalSteps: 1000
		}));
		expect(get(progress)).toBe(1);
	});

	it('updates reactively', () => {
		workerState.update((s) => ({ ...s, currentStep: 250, totalSteps: 1000 }));
		expect(get(progress)).toBe(0.25);

		workerState.update((s) => ({ ...s, currentStep: 750 }));
		expect(get(progress)).toBe(0.75);
	});
});

describe('resetWorkerState', () => {
	it('resets all state to defaults', () => {
		workerState.update((s) => ({
			...s,
			isRunning: true,
			currentStep: 5000,
			speed: 1000,
			winnerCount: 3
		}));
		addEvent({ type: 'info', message: 'test' });

		resetWorkerState();

		const state = get(workerState);
		expect(state.isRunning).toBe(false);
		expect(state.currentStep).toBe(0);
		expect(state.speed).toBe(0);
		expect(state.winnerCount).toBe(0);
		expect(get(events)).toEqual([]);
	});
});
