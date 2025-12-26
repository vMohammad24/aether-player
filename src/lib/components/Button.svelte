<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { twMerge } from 'tailwind-merge';

	let {
		leftIcon = undefined,
		rightIcon = undefined,
		type = 'button',
		variant = 'primary',
		size = 'md',
		href = undefined,
		disabled = false,
		children,
		class: userClass = undefined,
		...rest
	} = $props<{
		leftIcon?: Component;
		rightIcon?: Component;
		type?: 'button' | 'submit' | 'reset';
		variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive';
		size?: 'sm' | 'md' | 'lg';
		href?: string;
		disabled?: boolean;
		children?: Snippet;
		class?: string;
		[key: string]: any;
	}>();

	const baseClasses =
		'cursor-pointer inline-flex items-center justify-center rounded-md font-semibold transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-background';

	const sizeStyles: Record<'sm' | 'md' | 'lg', string> = {
		sm: 'px-3 py-1.5 text-sm gap-1.5',
		md: 'px-4 py-2 text-base gap-2',
		lg: 'px-6 py-3 text-lg gap-2.5'
	};

	const variantStyles: Record<
		'primary' | 'secondary' | 'outline' | 'ghost' | 'destructive',
		string
	> = {
		primary: 'bg-primary text-text hover:bg-secondary focus:ring-primary',
		secondary: 'bg-secondary text-text hover:bg-primary focus:ring-secondary',
		outline: 'border border-primary text-text hover:bg-primary focus:ring-primary',
		ghost: 'text-text hover:bg-primary hover:bg-opacity-20 focus:ring-primary',
		destructive: 'bg-red text-white hover:bg-red/80 focus:ring-red'
	};

	const disabledClasses = 'opacity-50 cursor-not-allowed';

	const iconSizeClasses = {
		sm: 'h-4 w-4',
		md: 'h-5 w-5',
		lg: 'h-6 w-6'
	};
	const iconBaseClass = 'inline-block';

	const currentIconSizeClass = $derived(iconSizeClasses[size as 'sm' | 'md' | 'lg']);

	const finalClasses = $derived(
		twMerge(
			[
				baseClasses,
				sizeStyles[size as 'sm' | 'md' | 'lg'],
				variantStyles[variant as 'primary' | 'secondary' | 'outline' | 'ghost'],
				disabled ? disabledClasses : '',
				userClass
			]
				.filter(Boolean)
				.map((cls) => cls.trim())
		)
	);

	const Tag = $derived(href ? 'a' : 'button');
	const finalHref = $derived(Tag === 'a' && !disabled ? href : undefined);
	const buttonType = $derived(Tag === 'button' ? type : undefined);

	function handleClick(event: MouseEvent) {
		if (disabled) {
			event.preventDefault();
			event.stopImmediatePropagation();
			return;
		}
		rest.onclick?.(event);
	}
</script>

<svelte:element
	this={Tag}
	{...Tag === 'a' ? { href: finalHref } : {}}
	{...Tag === 'button' ? { type: buttonType, disabled: disabled } : {}}
	class={finalClasses}
	aria-disabled={disabled ? 'true' : undefined}
	role={Tag === 'a' ? 'button' : undefined}
	onclick={handleClick}
	{...rest}
>
	{#if leftIcon}
		{@render leftIcon({ class: twMerge(iconBaseClass, currentIconSizeClass) })}
	{/if}
	{#if children}
		{@render children()}
	{/if}
	{#if rightIcon}
		{@render rightIcon({ class: twMerge(iconBaseClass, currentIconSizeClass) })}
	{/if}
</svelte:element>
