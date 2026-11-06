import { writable } from 'svelte/store';

// Shared between the window shell and Appearance so native failures stay visible.
export const materialError = writable('');
