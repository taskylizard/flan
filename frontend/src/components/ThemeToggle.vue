<script setup lang="ts">
import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useColorMode } from '@vueuse/core'

const mode = useColorMode()
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button aria-label="Toggle Theme" variant="outline" size="icon">
        <IRadixSun class="size-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
        <IRadixMoon
          class="absolute size-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
        />
        <span class="sr-only">Toggle theme</span>
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent align="end" class="min-w-32">
      <DropdownMenuItem @click="mode = 'light'">
        <IRadixSun class="mr-2 size-4" />
        Light
      </DropdownMenuItem>
      <DropdownMenuItem @click="mode = 'dark'">
        <IRadixMoon class="mr-2 size-4" />
        Dark
      </DropdownMenuItem>
      <DropdownMenuItem @click="mode = 'auto'">
        <IRadixMobile class="mr-2 size-4" />
        System
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</template>
