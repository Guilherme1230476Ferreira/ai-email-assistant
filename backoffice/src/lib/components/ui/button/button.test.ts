import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import Button from './button.svelte';

describe('Button Component', () => {
	it('renders as a button by default', () => {
		render(Button, { props: { 'data-testid': 'btn' } });
		
		const button = screen.getByTestId('btn');
		expect(button).toBeInTheDocument();
		expect(button.tagName).toBe('BUTTON');
		expect(button).toHaveClass('bg-primary'); // Default variant class
	});

	it('applies variant classes correctly', () => {
		render(Button, { props: { variant: 'destructive', 'data-testid': 'btn' } });
		
		const button = screen.getByTestId('btn');
		expect(button).toHaveClass('text-destructive');
	});

	it('renders as an anchor tag when href is provided', () => {
		render(Button, { props: { href: '/login', 'data-testid': 'link' } });
		
		const link = screen.getByTestId('link');
		expect(link).toBeInTheDocument();
		expect(link.tagName).toBe('A');
		expect(link).toHaveAttribute('href', '/login');
	});

	it('handles click events', async () => {
		const onClick = vi.fn();
		render(Button, { props: { 'data-testid': 'btn', onclick: onClick } });

		const button = screen.getByTestId('btn');
		await fireEvent.click(button);
		
		expect(onClick).toHaveBeenCalledTimes(1);
	});

	it('is disabled when disabled prop is true', () => {
		render(Button, { props: { disabled: true, 'data-testid': 'btn' } });
		
		const button = screen.getByTestId('btn');
		expect(button).toBeDisabled();
	});
});
