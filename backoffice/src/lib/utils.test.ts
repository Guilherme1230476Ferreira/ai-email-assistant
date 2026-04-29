import { describe, it, expect } from 'vitest';
import { cn } from './utils';

describe('utils', () => {
	describe('cn()', () => {
		it('merges css classes correctly', () => {
			expect(cn('bg-red-500', 'text-white')).toBe('bg-red-500 text-white');
		});

		it('handles conditional classes', () => {
			expect(cn('bg-red-500', true && 'text-white', false && 'hidden')).toBe(
				'bg-red-500 text-white'
			);
		});

		it('merges tailwind classes intelligently', () => {
			// twMerge should resolve conflicts by taking the last class
			expect(cn('p-4 px-2', 'p-8')).toBe('p-8');
			expect(cn('bg-red-500', 'bg-blue-500')).toBe('bg-blue-500');
		});
	});
});
