<script setup lang="ts">
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { Toaster } from '@/components/ui/toast'
import { formatDate } from '@/lib/date'
import { useUserStore } from '@/stores/user'
import {
  Copy,
  ExternalLink,
  Image as ImageIcon,
  Loader2,
  Trash2,
  Upload,
  X,
} from 'lucide-vue-next'
import { useToast } from '../components/ui/toast/use-toast'

interface UrlFormat {
  width?: number
  height?: number
  quality?: number
  format?: 'webp' | 'jpeg' | 'png'
}

function generateImageUrl(baseUrl: string, options?: UrlFormat) {
  const url = new URL(baseUrl, window.location.origin)

  if (options?.width)
    url.searchParams.set('width', options.width.toString())
  if (options?.height)
    url.searchParams.set('height', options.height.toString())
  if (options?.quality)
    url.searchParams.set('quality', options.quality.toString())
  if (options?.format)
    url.searchParams.set('format', options.format)

  return url.toString()
}

interface UploadedImage {
  file_id: string
  url: string
  created_at: string
  isLoading?: boolean
  previewError?: boolean
  isDeleting?: boolean
}

interface State {
  selectedFile: File | null
  previewUrl: string
  isDragging: boolean
  isUploading: boolean
  isLoading: boolean
  uploadError: string
  images: UploadedImage[]
  fileInput: HTMLInputElement | null
  showDeleteDialog: boolean
  imageToDelete: UploadedImage | null
}

const { toast } = useToast()
const userStore = useUserStore()

const state = reactive<State>({
  selectedFile: null,
  previewUrl: '',
  isDragging: false,
  isUploading: false,
  isLoading: false,
  uploadError: '',
  images: [],
  fileInput: null,
  showDeleteDialog: false,
  imageToDelete: null,
})

function handleFileSelect(event: Event) {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (file) {
    processSelectedFile(file)
  }
}

function handleDrop(event: DragEvent) {
  state.isDragging = false
  const file = event.dataTransfer?.files[0]
  if (file && file.type.startsWith('image/')) {
    processSelectedFile(file)
  }
  else {
    toast({
      title: 'Invalid File',
      description: 'Please drop an image file',
      variant: 'destructive',
    })
  }
}

function processSelectedFile(file: File) {
  state.selectedFile = file
  state.previewUrl = URL.createObjectURL(file)
  state.uploadError = ''
}

function cancelUpload() {
  state.selectedFile = null
  state.previewUrl = ''
  state.uploadError = ''
  if (state.fileInput) {
    state.fileInput.value = ''
  }
}

const { isSupported, copy, copied } = useClipboard()

watch(copied, (copied) => {
  if (copied) {
    toast({
      title: 'Copied!',
      description: 'URL copied to clipboard',
    })
  }
})

async function copyImageUrl(url: string, formatName: string) {
  await copy(url)
  toast({
    title: 'Copied!',
    description: `${formatName} URL copied to clipboard`,
  })
}

async function uploadFile() {
  if (!state.selectedFile || !userStore.isAuthenticated)
    return

  state.isUploading = true
  state.uploadError = ''

  const formData = new FormData()
  formData.append('file', state.selectedFile)

  try {
    const { data, error } = await useFetch('/api/upload', {
      method: 'POST',
      body: formData,
      headers: {
        'X-Username': userStore.username!,
        'X-Access-Key': userStore.accessKey!,
      },
    }).json<any>()

    if (error.value) {
      throw new Error(error.value as string)
    }

    if (data.value) {
      await fetchImages()
      toast({
        title: 'Success',
        description: 'Image uploaded successfully',
      })
      cancelUpload()
    }
  }
  catch (err) {
    toast({
      title: 'Upload Failed',
      description:
        err instanceof Error
          ? err.message
          : 'An error occurred while uploading',
      variant: 'destructive',
    })
  }
  finally {
    state.isUploading = false
  }
}

async function deleteImage(image: UploadedImage) {
  if (!userStore.isAuthenticated || !image.file_id)
    return

  // Set deleting state for the specific image
  const targetImage = state.images.find(img => img.file_id === image.file_id)
  if (targetImage) {
    targetImage.isDeleting = true
  }

  try {
    const { error } = await useFetch(`/api/delete/${image.file_id}`, {
      method: 'DELETE',
      headers: {
        'X-Username': userStore.username!,
        'X-Access-Key': userStore.accessKey!,
      },
    })

    if (error.value) {
      throw new Error(error.value as string)
    }

    // Remove the image from the list
    state.images = state.images.filter(img => img.file_id !== image.file_id)

    toast({
      title: 'Success',
      description: 'Image deleted successfully',
    })
  }
  catch (err) {
    toast({
      title: 'Delete Failed',
      description:
        err instanceof Error ? err.message : 'Failed to delete image',
      variant: 'destructive',
    })
  }
  finally {
    const targetImage = state.images.find(
      img => img.file_id === image.file_id,
    )
    if (targetImage) {
      targetImage.isDeleting = false
    }
    state.showDeleteDialog = false
    state.imageToDelete = null
  }
}

function confirmDelete(image: UploadedImage) {
  state.imageToDelete = image
  state.showDeleteDialog = true
}

function handleImageLoad(image: UploadedImage) {
  image.isLoading = false
  image.previewError = false
}

function handleImageError(image: UploadedImage) {
  image.isLoading = false
  image.previewError = true
}

async function fetchImages() {
  if (
    !userStore.isAuthenticated
    || !userStore.username
    || !userStore.accessKey
  ) {
    state.images = []
    return
  }

  // Prevent multiple concurrent fetches
  if (state.isLoading)
    return

  state.isLoading = true
  state.images = [] // Clear previous images

  try {
    const { data, error } = await useFetch('/api/list', {
      headers: {
        'X-Username': userStore.username,
        'X-Access-Key': userStore.accessKey,
      },
    }).json()

    if (error.value) {
      throw new Error(error.value as string)
    }

    if (data.value) {
      const images = data.value
        ? (data.value as { images: UploadedImage[] }).images
        : []

      state.images = images.map(img => ({
        url: img.url,
        file_id: img.file_id,
        created_at: img.created_at,
        isLoading: true,
        previewError: false,
      }))
    }
  }
  catch (err) {
    toast({
      title: 'Failed to Load Images',
      description:
        err instanceof Error ? err.message : 'Failed to load image history',
      variant: 'destructive',
    })
  }
  finally {
    state.isLoading = false
  }
}

tryOnMounted(() => {
  if (userStore.isAuthenticated) {
    fetchImages()
  }
})

watch(
  () => userStore.isAuthenticated,
  (isAuthenticated) => {
    if (isAuthenticated) {
      fetchImages()
    }
    else {
      state.images = []
    }
  },
)
</script>

<template>
  <div class="mx-auto max-w-4xl p-6">
    <Card>
      <CardContent class="pt-6">
        <div
          class="rounded-lg border-2 border-dashed p-8 text-center"
          :class="{
            'border-primary bg-primary/5': state.isDragging,
            'border-muted-foreground': !state.isDragging,
          }"
          @dragenter.prevent="state.isDragging = true"
          @dragover.prevent="state.isDragging = true"
          @dragleave.prevent="state.isDragging = false"
          @drop.prevent="handleDrop"
        >
          <div v-if="!state.selectedFile && !state.isUploading">
            <Upload class="mx-auto mb-4 size-12 text-muted-foreground" />
            <h3 class="mb-2 text-lg font-medium">
              Drop your image here
            </h3>
            <p class="mb-4 text-sm text-muted-foreground">
              Or click to browse
            </p>
            <label
              class="inline-block cursor-pointer"
              tabindex="0"
              @keydown.enter="state.fileInput?.click()"
            >
              <Button variant="secondary" as="span">Choose File</Button>
              <input
                type="file"
                accept="image/*"
                class="hidden"
                @change="handleFileSelect"
              >
            </label>
          </div>

          <div v-else-if="state.isUploading" class="space-y-4">
            <Loader2 class="mx-auto size-8 animate-spin" />
            <p class="text-sm text-muted-foreground">
              Uploading your image...
            </p>
          </div>

          <div v-else class="space-y-4">
            <img
              :src="state.previewUrl"
              alt="Preview"
              class="mx-auto max-h-48 rounded-lg"
            >
            <Button :disabled="state.isUploading" @click="uploadFile">
              <Upload class="mr-2 size-4" />
              Upload
            </Button>
            <Button variant="outline" @click="cancelUpload">
              <X class="mr-2 size-4" />
              Cancel
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- Image History -->
    <Card class="mt-6">
      <CardHeader>
        <CardTitle class="text-lg">
          History
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div v-if="state.isLoading" class="flex justify-center py-4">
          <Loader2 class="size-6 animate-spin" />
        </div>

        <div
          v-else-if="state.images.length > 0"
          class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3"
        >
          <Card
            v-for="image in state.images"
            :key="image.file_id"
            class="group overflow-hidden"
          >
            <div class="relative aspect-square">
              <!-- Loading state -->
              <div
                v-if="image.isLoading"
                class="absolute inset-0 flex items-center justify-center bg-muted"
              >
                <Loader2 class="size-6 animate-spin" />
              </div>

              <!-- Error state -->
              <div
                v-else-if="image.previewError"
                class="absolute inset-0 flex items-center justify-center bg-muted"
              >
                <ImageIcon class="size-8 text-muted-foreground" />
              </div>

              <!-- Image preview -->
              <img
                :src="generateImageUrl(image.url, {
                  width: 400,
                  height: 400,
                  format: 'webp',
                  quality: 80,
                })"
                :alt="`Uploaded image ${image.file_id}`"
                class="size-full object-cover"
                @load="() => handleImageLoad(image)"
                @error="() => handleImageError(image)"
              >

              <!-- Overlay -->
              <div
                class="absolute inset-0 flex items-center justify-center gap-2 bg-popover/80 opacity-0 transition-opacity group-hover:opacity-100"
              >
                <DropdownMenu v-if="isSupported">
                  <DropdownMenuTrigger as-child>
                    <Button variant="secondary" size="icon">
                      <Copy class="size-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" class="w-48">
                    <DropdownMenuItem
                      @click="copyImageUrl(image.url, 'Original')"
                    >
                      Copy Original URL
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      @click="
                        copyImageUrl(
                          generateImageUrl(image.url, {
                            width: 800,
                            height: 800,
                            format: 'webp',
                            quality: 80,
                          }),
                          'Large WebP',
                        )
                      "
                    >
                      Copy Large WebP (800x800)
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      @click="
                        copyImageUrl(
                          generateImageUrl(image.url, {
                            width: 400,
                            height: 400,
                            format: 'webp',
                            quality: 80,
                          }),
                          'Medium WebP',
                        )
                      "
                    >
                      Copy Medium WebP (400x400)
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      @click="
                        copyImageUrl(
                          generateImageUrl(image.url, {
                            width: 200,
                            height: 200,
                            format: 'webp',
                            quality: 80,
                          }),
                          'Thumbnail WebP',
                        )
                      "
                    >
                      Copy Thumbnail WebP (200x200)
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      @click="
                        copyImageUrl(
                          generateImageUrl(image.url, {
                            width: 800,
                            height: 800,
                            format: 'jpeg',
                            quality: 90,
                          }),
                          'High Quality JPEG',
                        )
                      "
                    >
                      Copy High Quality JPEG
                    </DropdownMenuItem>
                    <DropdownMenuItem
                      @click="
                        copyImageUrl(
                          generateImageUrl(image.url, {
                            width: 800,
                            height: 800,
                            format: 'png',
                          }),
                          'PNG',
                        )
                      "
                    >
                      Copy PNG
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
                <a :href="image.url" target="_blank">
                  <Button variant="secondary" size="icon">
                    <ExternalLink class="size-4" />
                  </Button>
                </a>
                <Button
                  variant="destructive"
                  size="icon"
                  :disabled="image.isDeleting"
                  @click="() => confirmDelete(image)"
                >
                  <Loader2
                    v-if="image.isDeleting"
                    class="size-4 animate-spin"
                  />
                  <Trash2 v-else class="size-4" />
                </Button>
              </div>
            </div>
            <CardContent class="p-3">
              <p class="text-xs text-muted-foreground">
                {{ formatDate(image.created_at) }}
              </p>
            </CardContent>
          </Card>
        </div>

        <div v-else class="py-4 text-center text-muted-foreground">
          No images uploaded yet.
        </div>
      </CardContent>
    </Card>
    <!-- Delete Confirmation Dialog -->
    <AlertDialog
      :open="state.showDeleteDialog"
      @update:open="state.showDeleteDialog = $event"
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This action cannot be undone. This will permanently delete the
            image.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="state.showDeleteDialog = false">
            Cancel
          </AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            @click="state.imageToDelete && deleteImage(state.imageToDelete)"
          >
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
    <Toaster />
  </div>
</template>
