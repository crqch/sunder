<script lang="ts">
  import { ChevronLeft, ChevronRight, Calendar as CalendarIcon } from '@lucide/svelte';
  import { fly } from 'svelte/transition';

  let {
    mode = $bindable('range'),
    startDate = $bindable(null),
    endDate = $bindable(null)
  } = $props<{
    mode?: 'single' | 'range';
    startDate?: Date | null;
    endDate?: Date | null;
  }>();

  let open = $state(false);
  let containerRef: HTMLDivElement;

  let currentMonth = $state(new Date().getMonth());
  let currentYear = $state(new Date().getFullYear());

  let hoverDate = $state<Date | null>(null);

  const daysInMonth = $derived(new Date(currentYear, currentMonth + 1, 0).getDate());
  const firstDayOfMonth = $derived(new Date(currentYear, currentMonth, 1).getDay()); // 0 = Sunday

  let days = $derived.by(() => {
    let arr = [];
    for (let i = 0; i < firstDayOfMonth; i++) arr.push(null);
    for (let i = 1; i <= daysInMonth; i++) arr.push(new Date(currentYear, currentMonth, i));
    return arr;
  });

  const monthNames = [
    'January',
    'February',
    'March',
    'April',
    'May',
    'June',
    'July',
    'August',
    'September',
    'October',
    'November',
    'December'
  ];

  function handleDateClick(d: Date | null) {
    if (!d) return;

    // Normalize clicked date to midnight
    let clicked = new Date(d);
    clicked.setHours(0, 0, 0, 0);

    if (mode === 'single') {
      startDate = clicked;
      endDate = clicked;
      open = false;
    } else {
      if (startDate && endDate) {
        let t = clicked.getTime();
        if (t === startDate.getTime()) {
          // Clicked start date, keep end date as anchor
          startDate = endDate;
          endDate = null;
        } else if (t === endDate.getTime()) {
          // Clicked end date, keep start date as anchor
          endDate = null;
        } else {
          // Start entirely new range
          startDate = clicked;
          endDate = null;
        }
      } else if (!startDate) {
        startDate = clicked;
        endDate = null;
      } else {
        if (clicked < startDate) {
          endDate = startDate;
          startDate = clicked;
        } else {
          endDate = clicked;
        }
      }
    }
  }

  function setToday() {
    let today = new Date();
    today.setHours(0, 0, 0, 0);
    currentMonth = today.getMonth();
    currentYear = today.getFullYear();

    if (mode === 'range' && startDate && endDate) {
      startDate = today;
      endDate = today;
    } else {
      handleDateClick(today);
    }
  }

  function setQuickPick(months: number) {
    let today = new Date();
    today.setHours(0, 0, 0, 0);
    let past = new Date(today);
    past.setMonth(past.getMonth() - months);
    startDate = past;
    endDate = today;
    currentMonth = past.getMonth();
    currentYear = past.getFullYear();
    viewMode = 'calendar';
  }

  let viewMode = $state<'calendar' | 'months' | 'years'>('calendar');
  let yearPageStart = $derived(Math.floor(currentYear / 12) * 12);

  function prevView() {
    if (viewMode === 'calendar') {
      if (currentMonth === 0) {
        currentMonth = 11;
        currentYear--;
      } else {
        currentMonth--;
      }
    } else if (viewMode === 'months') {
      currentYear--;
    } else if (viewMode === 'years') {
      currentYear -= 12;
    }
  }

  function nextView() {
    if (viewMode === 'calendar') {
      if (currentMonth === 11) {
        currentMonth = 0;
        currentYear++;
      } else {
        currentMonth++;
      }
    } else if (viewMode === 'months') {
      currentYear++;
    } else if (viewMode === 'years') {
      currentYear += 12;
    }
  }

  $effect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (open && containerRef && !containerRef.contains(event.target as Node)) {
        open = false;
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  });

  function formatDate(d: Date | null) {
    if (!d) return '';
    return d.toLocaleDateString();
  }

  let displayText = $derived(() => {
    if (!startDate) return 'Select date...';
    if (mode === 'single' || (startDate && endDate && startDate.getTime() === endDate.getTime()))
      return formatDate(startDate);
    if (startDate && !endDate) return `${formatDate(startDate)} - ...`;
    return `${formatDate(startDate)} - ${formatDate(endDate)}`;
  });

  function isSelected(d: Date | null) {
    if (!d) return false;
    let t = d.getTime();
    if (startDate && t === startDate.getTime()) return true;
    if (endDate && t === endDate.getTime()) return true;
    return false;
  }

  function isInRange(d: Date | null) {
    if (!d || !startDate) return false;
    let t = d.getTime();
    let e = endDate || hoverDate;
    if (!e) return false;
    let sTime = startDate.getTime();
    let eTime = e.getTime();
    if (sTime < eTime) return t > sTime && t < eTime;
    return t > eTime && t < sTime;
  }
</script>

<div class="relative w-full {open ? 'z-50' : ''}" bind:this={containerRef}>
  <button
    type="button"
    onclick={() => (open = !open)}
    class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex w-full items-center justify-between rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
  >
    <div class="flex items-center gap-2 truncate">
      <CalendarIcon size={16} class="text-muted-foreground shrink-0" />
      <span class="truncate">{displayText()}</span>
    </div>
  </button>

  {#if open}
    <div
      class="bg-card border-border/50 absolute right-0 z-50 mt-1 w-72 origin-top rounded-lg border p-3 shadow-lg md:right-auto md:left-0"
      transition:fly={{ y: -10, duration: 200 }}
    >
      {#if mode === 'range'}
        <div class="border-border/50 mb-3 grid grid-cols-4 gap-1.5 border-b pb-3">
          <button
            onclick={() => setQuickPick(1)}
            class="bg-muted hover:bg-primary/20 w-full rounded-md py-1.5 text-xs transition-colors"
            >1M</button
          >
          <button
            onclick={() => setQuickPick(3)}
            class="bg-muted hover:bg-primary/20 w-full rounded-md py-1.5 text-xs transition-colors"
            >3M</button
          >
          <button
            onclick={() => setQuickPick(6)}
            class="bg-muted hover:bg-primary/20 w-full rounded-md py-1.5 text-xs transition-colors"
            >6M</button
          >
          <button
            onclick={() => setQuickPick(12)}
            class="bg-muted hover:bg-primary/20 w-full rounded-md py-1.5 text-xs transition-colors"
            >1Y</button
          >
        </div>
      {/if}

      <div class="mb-4 flex items-center justify-between">
        <button onclick={prevView} class="hover:bg-muted rounded-md p-1"
          ><ChevronLeft size={16} /></button
        >

        <div class="flex items-center gap-1">
          {#if viewMode === 'calendar'}
            <button
              onclick={() => (viewMode = 'months')}
              class="hover:bg-muted rounded-md px-2 py-1 text-sm font-medium transition-colors"
              >{monthNames[currentMonth]}</button
            >
            <button
              onclick={() => (viewMode = 'years')}
              class="hover:bg-muted rounded-md px-2 py-1 text-sm font-medium transition-colors"
              >{currentYear}</button
            >
          {:else if viewMode === 'months'}
            <button
              onclick={() => (viewMode = 'years')}
              class="hover:bg-muted rounded-md px-2 py-1 text-sm font-medium transition-colors"
              >{currentYear}</button
            >
          {:else if viewMode === 'years'}
            <span class="px-2 py-1 text-sm font-medium">{yearPageStart} - {yearPageStart + 11}</span
            >
          {/if}
        </div>

        <button onclick={nextView} class="hover:bg-muted rounded-md p-1"
          ><ChevronRight size={16} /></button
        >
      </div>

      {#if viewMode === 'calendar'}
        <div
          class="text-muted-foreground mb-2 grid grid-cols-7 gap-1 text-center text-xs font-medium"
        >
          <div>Su</div>
          <div>Mo</div>
          <div>Tu</div>
          <div>We</div>
          <div>Th</div>
          <div>Fr</div>
          <div>Sa</div>
        </div>

        <div class="grid grid-cols-7">
          {#each days as d}
            {#if d === null}
              <div class="h-9 w-full"></div>
            {:else}
              {@const selected = isSelected(d)}
              {@const inRange = isInRange(d)}
              {@const sTime = startDate ? startDate.getTime() : 0}
              {@const eTime = endDate ? endDate.getTime() : hoverDate ? hoverDate.getTime() : 0}
              {@const isStart = startDate && d.getTime() === sTime}
              {@const isEnd =
                (endDate && d.getTime() === endDate.getTime()) ||
                (!endDate && hoverDate && d.getTime() === hoverDate.getTime())}

              <button
                type="button"
                onclick={() => handleDateClick(d)}
                onmouseenter={() => (hoverDate = d)}
                onmouseleave={() => (hoverDate = null)}
                class="group relative flex h-9 w-full items-center justify-center focus:outline-none"
              >
                {#if inRange}
                  <div class="bg-primary/20 absolute top-[2px] right-0 bottom-[2px] left-0"></div>
                {/if}
                {#if isStart && eTime > sTime}
                  <div class="bg-primary/20 absolute top-[2px] right-0 bottom-[2px] w-1/2"></div>
                {/if}
                {#if isEnd && eTime > sTime && d.getTime() === eTime && sTime !== 0}
                  <div class="bg-primary/20 absolute top-[2px] bottom-[2px] left-0 w-1/2"></div>
                {/if}
                {#if isStart && eTime < sTime && eTime !== 0}
                  <div class="bg-primary/20 absolute top-[2px] bottom-[2px] left-0 w-1/2"></div>
                {/if}
                {#if isEnd && eTime < sTime && d.getTime() === eTime}
                  <div class="bg-primary/20 absolute top-[2px] right-0 bottom-[2px] w-1/2"></div>
                {/if}

                <span
                  class="relative z-10 flex h-8 w-8 items-center justify-center rounded-md text-sm
                    {selected
                    ? 'bg-primary text-primary-foreground'
                    : 'group-hover:bg-primary group-hover:text-primary-foreground text-foreground'}"
                >
                  {d.getDate()}
                </span>
              </button>
            {/if}
          {/each}
        </div>
      {:else if viewMode === 'months'}
        <div class="grid grid-cols-3 gap-2 py-2">
          {#each monthNames as m, i}
            <button
              type="button"
              onclick={() => {
                currentMonth = i;
                viewMode = 'calendar';
              }}
              class="rounded-md p-2 text-sm transition-colors {i === currentMonth
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-muted text-foreground'}"
            >
              {m.substring(0, 3)}
            </button>
          {/each}
        </div>
      {:else if viewMode === 'years'}
        <div class="grid grid-cols-3 gap-2 py-2">
          {#each Array(12) as _, i}
            {@const y = yearPageStart + i}
            <button
              type="button"
              onclick={() => {
                currentYear = y;
                viewMode = 'months';
              }}
              class="rounded-md p-2 text-sm transition-colors {y === currentYear
                ? 'bg-primary text-primary-foreground'
                : 'hover:bg-muted text-foreground'}"
            >
              {y}
            </button>
          {/each}
        </div>
      {/if}

      <div class="border-border/50 mt-4 flex items-center justify-between border-t pt-3">
        <button onclick={setToday} class="text-primary text-xs font-medium hover:underline"
          >Today</button
        >
        {#if mode === 'range'}
          <button onclick={() => (open = false)} class="btn btn-sm">Done</button>
        {/if}
      </div>
    </div>
  {/if}
</div>
