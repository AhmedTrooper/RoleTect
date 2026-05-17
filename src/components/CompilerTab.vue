<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save, open as openDialog } from '@tauri-apps/plugin-dialog';
import { join } from '@tauri-apps/api/path';
import { 
  writeFile, 
  readDir, 
  readTextFile, 
  mkdir, 
  remove, 
  exists,
  stat
} from '@tauri-apps/plugin-fs';
import { Motion, AnimatePresence } from 'motion-v';
import { useSettingsStore } from '../store/settings';
import { useDialogStore } from '../store/dialog';
import { 
  Hammer, 
  Download, 
  Wand2, 
  RotateCw, 
  X,
  FileCode,
  Terminal,
  FolderOpen,
  File,
  ChevronRight,
  ChevronDown,
  Plus,
  Trash2,
  FolderPlus,
  Files,
  Save
} from '@lucide/vue';

// Codemirror imports
import { Codemirror } from 'vue-codemirror';
import { latex, latexLanguage, autoCloseTags } from 'codemirror-lang-latex';
import { oneDark } from '@codemirror/theme-one-dark';
import { EditorView } from '@codemirror/view';

const settingsStore = useSettingsStore();
const dialog = useDialogStore();

// Codemirror Extensions
const extensions = [
  latex(),
  latexLanguage,
  ...autoCloseTags,
  oneDark,
  EditorView.lineWrapping
];

// Types
interface FileItem {
  name: string;
  path: string;
  isDir: boolean;
  children?: FileItem[];
  isOpen?: boolean;
}

// State
const workspacePath = ref<string | null>(null);
const fileTree = ref<FileItem[]>([]);
const activeFilePath = ref<string | null>(null);
const latexCode = ref('');

const isSidebarVisible = ref(true);
const sidebarWidth = ref(240);
const isResizing = ref(false);

const pdfUrl = ref<string | null>(null);
const pdfBytesBuffer = ref<Uint8Array | null>(null);
const isCompiling = ref(false);
const isFixing = ref(false);
const isRefining = ref(false);
const refinementInstruction = ref('');
const isDownloading = ref(false);
const compilationError = ref<string | null>(null);
const isDirty = ref(false);
const editorContainer = ref<HTMLElement | null>(null);
const isLoadingWorkspace = ref(false);

// Tooltip State
const activeTooltip = ref<string | null>(null);

// Workspace Management
const toggleSidebar = () => {
  isSidebarVisible.value = !isSidebarVisible.value;
};

const startResizing = (e: MouseEvent) => {
  isResizing.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopResizing);
};

const handleMouseMove = (e: MouseEvent) => {
  if (!isResizing.value) return;
  const newWidth = e.clientX;
  if (newWidth > 150 && newWidth < 500) {
    sidebarWidth.value = newWidth;
  }
};

const stopResizing = () => {
  isResizing.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopResizing);
};

// Persistence & Initialization
onMounted(async () => {
  try {
    const savedWorkspace = await invoke<string | null>('get_workspace_path');
    if (savedWorkspace && await exists(savedWorkspace)) {
      workspacePath.value = savedWorkspace;
      await refreshFileTree();

      const lastFile = await invoke<string | null>('get_last_opened_file');
      if (lastFile && await exists(lastFile)) {
        await selectFile({ name: lastFile.split(/[/\\]/).pop() || '', path: lastFile, isDir: false });
      }
    } else {
      const savedCode = await invoke<string | null>('get_compiler_state');
      if (savedCode) {
        latexCode.value = savedCode;
      }
    }
    // Ensure initial load doesn't mark it as dirty
    setTimeout(() => { isDirty.value = false; }, 100);
  } catch (err) {
    console.error('Failed to initialize compiler:', err);
  }
});

onUnmounted(async () => {
  if (isDirty.value) {
    await saveActiveFile();
  }
});

// Workspace Management
const selectWorkspace = async () => {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: 'Select LaTeX Workspace'
    });

    if (selected && typeof selected === 'string') {
      workspacePath.value = selected;
      await invoke('save_workspace_path', { path: selected });
      await refreshFileTree();
    }
  } catch (err) {
    console.error('Failed to select workspace:', err);
  }
};

const refreshFileTree = async () => {
  if (!workspacePath.value) return;
  isLoadingWorkspace.value = true;
  try {
    fileTree.value = await scanDirectory(workspacePath.value);
  } catch (err) {
    console.error('Failed to scan workspace:', err);
  } finally {
    isLoadingWorkspace.value = false;
  }
};

const scanDirectory = async (dir: string): Promise<FileItem[]> => {
  const entries = await readDir(dir);
  const items: FileItem[] = [];

  for (const entry of entries) {
    const fullPath = await join(dir, entry.name);
    const isDir = entry.isDirectory;
    
    items.push({
      name: entry.name,
      path: fullPath,
      isDir: isDir,
      isOpen: false,
      children: isDir ? [] : undefined
    });
  }

  // Sort: Directories first, then alphabetically
  return items.sort((a, b) => {
    if (a.isDir && !b.isDir) return -1;
    if (!a.isDir && b.isDir) return 1;
    return a.name.localeCompare(b.name);
  });
};

const toggleFolder = async (item: FileItem) => {
  item.isOpen = !item.isOpen;
  if (item.isOpen && item.children?.length === 0) {
    item.children = await scanDirectory(item.path);
  }
};

const selectFile = async (item: FileItem) => {
  if (item.isDir) return;
  
  if (isDirty.value && activeFilePath.value) {
    await saveActiveFile();
  }

  try {
    // Bulletproof: Check existence before reading
    const fileExists = await exists(item.path);
    if (!fileExists) {
      await dialog.showAlert(`The file "${item.name}" no longer exists on disk.`, 'File Not Found');
      await refreshFileTree();
      return;
    }

    const content = await readTextFile(item.path);
    latexCode.value = content;
    activeFilePath.value = item.path;
    isDirty.value = false;
    await invoke('save_last_opened_file', { path: item.path });
    pdfUrl.value = null; // Reset preview for new file
  } catch (err: any) {
    console.error('Failed to read file:', err);
    await dialog.showAlert(`Failed to open file: ${err.message || err.toString()}`, 'Read Error');
  }
};

const saveActiveFile = async () => {
  if (!activeFilePath.value) {
    // Fallback to standalone state
    await invoke('save_compiler_state', { latexContent: latexCode.value });
    return;
  }

  try {
    // Bulletproof: Check if parent directory still exists
    const dirPath = activeFilePath.value.substring(0, activeFilePath.value.lastIndexOf(/[/\\]/));
    if (dirPath && !(await exists(dirPath))) {
      await dialog.showAlert("The parent directory for this file is missing.", "Save Failed");
      return;
    }

    await writeFile(activeFilePath.value, new TextEncoder().encode(latexCode.value));
    isDirty.value = false;
  } catch (err: any) {
    console.error('Failed to save file:', err);
    await dialog.showAlert(`Save failed: ${err.message || err.toString()}`, 'Write Error');
  }
};

const createNewFile = async (parent: FileItem | null = null) => {
  const dir = parent ? parent.path : workspacePath.value;
  if (!dir) return;

  const fileName = await dialog.showPrompt('Enter file name (e.g. main.tex):', '', 'New File');
  if (!fileName) return;

  const fullPath = await join(dir, fileName);
  try {
    await writeFile(fullPath, new TextEncoder().encode(''));
    if (parent) {
      parent.isOpen = true;
      parent.children = await scanDirectory(parent.path);
    } else {
      await refreshFileTree();
    }
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Failed to create file');
  }
};

const createNewFolder = async (parent: FileItem | null = null) => {
  const dir = parent ? parent.path : workspacePath.value;
  if (!dir) return;

  const folderName = await dialog.showPrompt('Enter folder name:', '', 'New Folder');
  if (!folderName) return;

  const fullPath = await join(dir, folderName);
  try {
    await mkdir(fullPath);
    if (parent) {
      parent.isOpen = true;
      parent.children = await scanDirectory(parent.path);
    } else {
      await refreshFileTree();
    }
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Failed to create folder');
  }
};

const deleteItem = async (item: FileItem) => {
  const confirmed = await dialog.showConfirm(`Are you sure you want to delete "${item.name}"?`, 'Delete Item');
  if (!confirmed) return;

  try {
    await remove(item.path, { recursive: item.isDir });
    
    // Bulletproof: If the active file (or its parent directory) was deleted, clear the editor
    if (activeFilePath.value) {
      const isSelf = activeFilePath.value === item.path;
      const isParent = activeFilePath.value.startsWith(item.path + '/') || activeFilePath.value.startsWith(item.path + '\\');
      
      if (isSelf || isParent) {
        activeFilePath.value = null;
        latexCode.value = '';
        isDirty.value = false;
        pdfUrl.value = null;
      }
    }
    
    await refreshFileTree();
  } catch (err: any) {
    await dialog.showAlert(err.toString(), 'Failed to delete item');
  }
};

const closeWorkspace = async () => {
  const confirmed = await dialog.showConfirm('Close workspace and return to standalone mode?', 'Close Workspace');
  if (!confirmed) return;

  workspacePath.value = null;
  fileTree.value = [];
  activeFilePath.value = null;
  await invoke('save_workspace_path', { path: '' });
  
  const savedCode = await invoke<string | null>('get_compiler_state');
  if (savedCode) {
    latexCode.value = savedCode;
  }
};

// Auto-save & Compile logic
watch(latexCode, () => {
  isDirty.value = true;
});

const handleBlur = async () => {
  if (isDirty.value) {
    if (settingsStore.isAutoCompileEnabled) {
      // compilePdf internally calls and awaits saveActiveFile()
      await compilePdf();
    } else {
      await saveActiveFile();
    }
  }
};

// AI Refinement
const refineWithAi = async () => {
  if (!latexCode.value || !refinementInstruction.value.trim() || isRefining.value) return;
  
  isRefining.value = true;
  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");

    const provider = settingsStore.selectedAiProvider;
    const model = settingsStore.selectedAiModel;

    const refinedCode = await invoke<string>('refine_latex_with_ai', {
      provider,
      model,
      apiKey,
      currentLatex: latexCode.value,
      instruction: refinementInstruction.value.trim()
    });

    latexCode.value = refinedCode;
    refinementInstruction.value = '';
    await saveActiveFile();
    await compilePdf();
  } catch (err: any) {
    console.error("AI Refinement Error:", err);
    await dialog.showAlert(err.toString(), 'AI Refinement Failed');
  } finally {
    isRefining.value = false;
  }
};

// Compile PDF
const compilePdf = async () => {
  if (!latexCode.value.trim()) return;
  
  isCompiling.value = true;
  compilationError.value = null;
  
  try {
    // Bulletproof: Force save before compile so the disk is in sync with the editor
    await saveActiveFile();

    let pdfBytes: number[];
    
    if (workspacePath.value && activeFilePath.value) {
      // Workspace-aware compilation
      let relativePath = activeFilePath.value.replace(workspacePath.value, '');
      if (relativePath.startsWith('/') || relativePath.startsWith('\\')) {
        relativePath = relativePath.substring(1);
      }

      pdfBytes = await invoke<number[]>('compile_workspace_to_pdf', { 
        workspaceDir: workspacePath.value,
        mainFileName: relativePath
      });
    } else {
      // Standalone string compilation
      pdfBytes = await invoke<number[]>('compile_resume_to_pdf', { 
        latexCode: latexCode.value 
      });
    }
    
    pdfBytesBuffer.value = new Uint8Array(pdfBytes);
    const blob = new Blob([pdfBytesBuffer.value], { type: 'application/pdf' });
    
    // Clean up previous URL to prevent memory leaks
    if (pdfUrl.value) {
      URL.revokeObjectURL(pdfUrl.value);
    }
    pdfUrl.value = URL.createObjectURL(blob);
  } catch (err: any) {
    console.error("Compilation Error:", err);
    compilationError.value = err.message || err.toString();
  } finally {
    isCompiling.value = false;
  }
};

// AI Fix
const fixWithAi = async () => {
  if (!latexCode.value || !compilationError.value || isFixing.value) return;
  
  isFixing.value = true;
  try {
    const apiKey = await settingsStore.getDecryptedKey();
    if (!apiKey) throw new Error("API Key not found. Please set it in Settings.");

    const provider = settingsStore.selectedAiProvider;
    const model = settingsStore.selectedAiModel;

    const fixedCode = await invoke<string>('fix_latex_with_ai', {
      provider,
      model,
      apiKey,
      brokenLatex: latexCode.value,
      errorLogs: compilationError.value
    });

    latexCode.value = fixedCode;
    compilationError.value = null;
    await saveActiveFile();
    await compilePdf();
  } catch (err: any) {
    console.error("AI Fix Error:", err);
    await dialog.showAlert(err.toString(), 'AI Fix Failed');
  } finally {
    isFixing.value = false;
  }
};

// Download PDF
const downloadPdf = async () => {
  if (!pdfBytesBuffer.value) return;
  isDownloading.value = true;
  
  try {
    const now = new Date();
    const timestamp = `${now.getFullYear()}${(now.getMonth() + 1).toString().padStart(2, '0')}${now.getDate().toString().padStart(2, '0')}_${now.getHours().toString().padStart(2, '0')}${now.getMinutes().toString().padStart(2, '0')}${now.getSeconds().toString().padStart(2, '0')}`;
    const defaultName = activeFilePath.value 
      ? activeFilePath.value.split(/[/\\]/).pop()?.replace('.tex', '.pdf') || `doc_${timestamp}.pdf`
      : `document_${timestamp}.pdf`;

    const filePath = await save({
      filters: [{ name: 'PDF Document', extensions: ['pdf'] }],
      defaultPath: defaultName
    });

    if (filePath) {
      await writeFile(filePath, pdfBytesBuffer.value);
      const filename = filePath.split(/[/\\]/).pop() || defaultName;
      await invoke('record_download', {
        filename,
        downloadType: 'compiler',
        jobId: null,
        contentId: null
      });
      await dialog.showAlert('PDF downloaded successfully.', 'Success');
    }
  } catch (err: any) {
    console.error("Download Error:", err);
    await dialog.showAlert(err.toString(), 'Download Failed');
  } finally {
    isDownloading.value = false;
  }
};

const activeFileName = computed(() => {
  if (!activeFilePath.value) return 'unsaved.tex';
  return activeFilePath.value.split(/[/\\]/).pop() || 'file.tex';
});
</script>

<template>
  <div class="compiler-container">
    <!-- Loading Overlay -->
    <AnimatePresence>
      <Motion
        v-if="isCompiling || isFixing || isRefining"
        :initial="{ opacity: 0 }"
        :animate="{ opacity: 1 }"
        :exit="{ opacity: 0 }"
        class="loading-overlay"
      >
        <div class="loader-content">
          <RotateCw :size="48" class="spinner" />
          <h2>{{ isFixing ? 'AI DEBUGGING...' : isRefining ? 'AI REFINING...' : 'COMPILING LATEX...' }}</h2>
          <p>Please wait while the engine processes your code.</p>
        </div>
      </Motion>
    </AnimatePresence>

    <header class="compiler-header">
      <div class="header-left">
        <button class="toggle-sidebar-btn" @click="toggleSidebar" title="Toggle Sidebar">
          <Layout :size="18" />
        </button>
        <Files :size="20" class="header-icon" />
        <h1>LaTeX IDE</h1>
        <span v-if="workspacePath" class="workspace-label">
          {{ workspacePath.split(/[/\\]/).pop() }}
        </span>
      </div>
      
      <div class="header-actions">
        <label class="auto-compile-toggle">
          <input 
            type="checkbox" 
            :checked="settingsStore.isAutoCompileEnabled"
            @change="settingsStore.setAutoCompile(($event.target as HTMLInputElement).checked)"
          >
          <span>Auto Compile</span>
        </label>
        
        <button 
          v-if="isDirty"
          class="action-btn save-btn"
          @click="saveActiveFile"
        >
          <Save :size="16" />
          <span>Save</span>
        </button>
        
        <button 
          v-if="compilationError" 
          class="action-btn ai-btn" 
          @click="fixWithAi" 
          :disabled="isFixing"
        >
          <Wand2 :size="16" />
          <span>AI Fix</span>
        </button>
        
        <button 
          class="action-btn compile-btn" 
          @click="compilePdf" 
          :disabled="isCompiling || !latexCode"
        >
          <Hammer :size="16" />
          <span>Compile</span>
        </button>
        
        <button 
          v-if="pdfUrl" 
          class="action-btn download-btn" 
          @click="downloadPdf" 
          :disabled="isDownloading"
        >
          <Download :size="16" />
          <span>Download</span>
        </button>
      </div>
    </header>

    <main class="compiler-main">
      <div class="split-pane">
        <!-- Sidebar File Explorer -->
        <aside v-if="isSidebarVisible" class="workspace-sidebar" :style="{ width: sidebarWidth + 'px' }">
          <div class="sidebar-header">
            <span>EXPLORER</span>
            <div class="header-tools">
              <button @click="refreshFileTree" title="Refresh"><RotateCw :size="12" /></button>
              <button @click="createNewFile()" title="New File"><Plus :size="14" /></button>
              <button @click="createNewFolder()" title="New Folder"><FolderPlus :size="14" /></button>
              <button v-if="workspacePath" @click="closeWorkspace" title="Close Workspace" class="close-workspace-btn"><X :size="14" /></button>
            </div>
          </div>

          <div v-if="!workspacePath" class="sidebar-empty">
            <FolderOpen :size="32" />
            <p>No workspace selected</p>
            <button class="btn-primary-sm" @click="selectWorkspace">Open Folder</button>
          </div>

          <div v-else class="file-tree">
            <div v-if="isLoadingWorkspace" class="tree-loading">
              <RotateCw :size="16" class="spinner" />
            </div>
            
            <template v-else>
              <div v-for="item in fileTree" :key="item.path" class="tree-item-wrapper">
                <div 
                  class="tree-item" 
                  :class="{ active: activeFilePath === item.path }"
                  @click="item.isDir ? toggleFolder(item) : selectFile(item)"
                >
                  <div class="item-icon">
                    <template v-if="item.isDir">
                      <ChevronRight v-if="!item.isOpen" :size="14" />
                      <ChevronDown v-else :size="14" />
                    </template>
                    <File v-else :size="14" />
                  </div>
                  <span class="item-name">{{ item.name }}</span>
                  <div class="item-actions">
                    <template v-if="item.isDir">
                      <button @click.stop="createNewFile(item)" title="New File"><Plus :size="12" /></button>
                      <button @click.stop="createNewFolder(item)" title="New Folder"><FolderPlus :size="12" /></button>
                    </template>
                    <button class="item-delete" @click.stop="deleteItem(item)" title="Delete"><Trash2 :size="12" /></button>
                  </div>
                </div>
                
                <AnimatePresence>
                  <Motion
                    v-if="item.isDir && item.isOpen"
                    :initial="{ height: 0, opacity: 0 }"
                    :animate="{ height: 'auto', opacity: 1 }"
                    :exit="{ height: 0, opacity: 0 }"
                    class="tree-children"
                  >
                    <div v-for="child in item.children" :key="child.path" class="tree-item-wrapper">
                      <div 
                        class="tree-item sub-item" 
                        :class="{ active: activeFilePath === child.path }"
                        @click="child.isDir ? toggleFolder(child) : selectFile(child)"
                      >
                        <div class="item-icon">
                          <template v-if="child.isDir">
                            <ChevronRight v-if="!child.isOpen" :size="14" />
                            <ChevronDown v-else :size="14" />
                          </template>
                          <File v-else :size="14" />
                        </div>
                        <span class="item-name">{{ child.name }}</span>
                        <div class="item-actions">
                          <template v-if="child.isDir">
                            <button @click.stop="createNewFile(child)" title="New File"><Plus :size="12" /></button>
                            <button @click.stop="createNewFolder(child)" title="New Folder"><FolderPlus :size="12" /></button>
                          </template>
                          <button class="item-delete" @click.stop="deleteItem(child)" title="Delete"><Trash2 :size="12" /></button>
                        </div>
                      </div>
                    </div>
                  </Motion>
                </AnimatePresence>
              </div>
            </template>
          </div>
        </aside>

        <!-- Sidebar Resizer -->
        <div v-if="isSidebarVisible" class="sidebar-resizer" @mousedown="startResizing"></div>

        <!-- Editor Section -->
        <section class="editor-section">
          <div class="pane-header">
            <div class="pane-header-left">
              <FileCode :size="14" />
              <span>{{ activeFileName }}</span>
              <span v-if="isDirty" class="dirty-indicator">●</span>
            </div>
            <div class="pane-header-actions" v-if="activeFilePath">
              <button 
                @click="saveActiveFile" 
                class="save-icon-btn" 
                :class="{ 'dirty': isDirty }" 
                title="Save Changes"
              >
                <Save :size="14" />
              </button>
            </div>
          </div>
          <div class="editor-relative-wrapper" ref="editorContainer">
            <codemirror
              v-model="latexCode"
              placeholder="Select a file or start typing here..."
              :style="{ height: '100%' }"
              :autofocus="true"
              :indent-with-tab="true"
              :tab-size="2"
              :extensions="extensions"
              @blur="handleBlur"
              class="latex-editor-cm"
            />

            <AnimatePresence>
              <Motion 
                v-if="latexCode"
                class="refinement-bar"
                drag
                :drag-constraints="editorContainer || undefined"
                :drag-elastic="0.1"
                :initial="{ opacity: 0, y: -10, x: '-50%' }"
                :animate="{ opacity: 1, y: 0, x: '-50%' }"
                :exit="{ opacity: 0, y: -10, x: '-50%' }"
              >
                <input 
                  v-model="refinementInstruction" 
                  placeholder="Refine code (e.g. 'Add a table of contents')..."
                  @keyup.enter="refineWithAi"
                />
                <button @click="refineWithAi" :disabled="isRefining">
                  {{ isRefining ? '...' : '→' }}
                </button>
              </Motion>
            </AnimatePresence>
          </div>
        </section>

        <!-- Preview Section -->
        <section class="preview-section">
          <div class="pane-header">
            <Terminal :size="14" />
            <span>PDF PREVIEW</span>
          </div>
          <div v-if="pdfUrl" class="pdf-viewer">
            <iframe :src="pdfUrl"></iframe>
          </div>
          <div v-else class="empty-preview">
            <div class="placeholder-content">
              <Hammer :size="48" />
              <h3>No PDF generated</h3>
              <p>Click "Compile" to generate a preview of your LaTeX code.</p>
            </div>
          </div>
        </section>
      </div>

      <!-- Error Console -->
      <AnimatePresence>
        <Motion
          v-if="compilationError"
          :initial="{ y: 100, opacity: 0 }"
          :animate="{ y: 0, opacity: 1 }"
          :exit="{ y: 100, opacity: 0 }"
          class="error-console"
        >
          <div class="console-header">
            <div class="title">
              <X :size="14" class="error-icon" />
              <span>COMPILATION ERROR</span>
            </div>
            <button class="close-btn" @click="compilationError = null">
              <X :size="14" />
            </button>
          </div>
          <pre class="error-logs">{{ compilationError }}</pre>
        </Motion>
      </AnimatePresence>
    </main>
  </div>
</template>

<style scoped>
.compiler-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg);
}

.compiler-header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toggle-sidebar-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 4px;
  transition: 0.15s;
}

.toggle-sidebar-btn:hover {
  background: var(--surface-soft);
  color: var(--ink);
}

.header-icon {
  color: var(--accent);
}

.header-left h1 {
  font-size: 0.9rem;
  font-weight: 700;
  color: var(--ink);
  margin: 0;
  letter-spacing: 0.02em;
}

.workspace-label {
  font-size: 0.7rem;
  background: var(--surface-soft);
  color: var(--muted);
  padding: 2px 8px;
  border-radius: 4px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.auto-compile-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--muted);
  cursor: pointer;
  user-select: none;
}

.auto-compile-toggle input {
  width: 14px;
  height: 14px;
  cursor: pointer;
  accent-color: var(--accent);
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: 700;
  cursor: pointer;
  transition: 0.2s;
  border: 1px solid var(--line);
  background: var(--surface-soft);
  color: var(--ink);
}

.action-btn:hover:not(:disabled) {
  border-color: var(--muted);
  background: var(--surface);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.compile-btn {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.compile-btn:hover:not(:disabled) {
  opacity: 0.9;
  background: var(--accent);
}

.save-btn {
  border-color: var(--accent-soft);
  background: var(--accent-soft);
  color: var(--accent);
}

.save-btn:hover:not(:disabled) {
  background: var(--accent);
  color: white;
  border-color: var(--accent);
}

.ai-btn {
  color: #a371f7;
  border-color: rgba(163, 113, 247, 0.3);
}

.ai-btn:hover:not(:disabled) {
  background: rgba(163, 113, 247, 0.1);
  border-color: #a371f7;
}

.compiler-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  position: relative;
}

.split-pane {
  flex: 1;
  display: flex;
  min-height: 0;
  position: relative;
}

.workspace-sidebar {
  background: var(--bg-accent);
  border-right: 1px solid var(--line);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  min-width: 150px;
  max-width: 500px;
}

.sidebar-resizer {
  width: 4px;
  cursor: col-resize;
  background: transparent;
  transition: background 0.2s;
  z-index: 10;
  margin-left: -2px;
}

.sidebar-resizer:hover, .sidebar-resizer:active {
  background: var(--accent);
}

.sidebar-header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--surface);
  border-bottom: 1px solid var(--line);
  font-size: 0.65rem;
  font-weight: 800;
  color: var(--muted);
  letter-spacing: 0.05em;
}

.header-tools {
  display: flex;
  gap: 8px;
}

.header-tools button {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
}

.header-tools button:hover {
  color: var(--ink);
}

.sidebar-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--muted);
  gap: 12px;
  padding: 20px;
  text-align: center;
}

.sidebar-empty p {
  font-size: 0.75rem;
  margin: 0;
}

.btn-primary-sm {
  background: var(--accent);
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
}

.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.tree-item-wrapper {
  display: flex;
  flex-direction: column;
}

.tree-item {
  display: flex;
  align-items: center;
  padding: 4px 12px;
  gap: 8px;
  cursor: pointer;
  transition: 0.1s;
  position: relative;
  user-select: none;
}

.tree-item:hover {
  background: var(--surface-soft);
}

.tree-item.active {
  background: var(--accent-soft);
  color: var(--accent);
}

.item-icon {
  display: flex;
  align-items: center;
  color: var(--muted);
}

.tree-item.active .item-icon {
  color: var(--accent);
}

.item-name {
  font-size: 0.8rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.item-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.1s;
}

.tree-item:hover .item-actions {
  opacity: 1;
}

.item-actions button {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
}

.item-actions button:hover {
  color: var(--ink);
}

.item-actions button.item-delete:hover {
  color: var(--warning);
}

.close-workspace-btn:hover {
  color: var(--warning) !important;
}

.tree-children {
  padding-left: 12px;
  overflow: hidden;
}

.tree-loading {
  display: flex;
  justify-content: center;
  padding: 20px;
  color: var(--accent);
}

.editor-section, .preview-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.editor-section {
  border-right: 1px solid var(--line);
}

.pane-header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: var(--bg-accent);
  border-bottom: 1px solid var(--line);
  font-size: 0.65rem;
  font-weight: 800;
  color: var(--muted);
  letter-spacing: 0.05em;
}

.pane-header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pane-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.save-icon-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.save-icon-btn:hover {
  background: var(--surface-soft);
  color: var(--ink);
}

.save-icon-btn.dirty {
  color: var(--accent);
}

.save-icon-btn.dirty:hover {
  background: var(--accent-soft);
}

.dirty-indicator {
  color: var(--accent);
  font-size: 10px;
  margin-left: -4px;
  text-shadow: 0 0 8px var(--accent);
  animation: pulse-dirty 2s infinite;
}

@keyframes pulse-dirty {
  0% { opacity: 0.6; }
  50% { opacity: 1; }
  100% { opacity: 0.6; }
}

.editor-relative-wrapper {
  flex: 1;
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: #282c34;
}

.latex-editor-cm {
  flex: 1;
  width: 100%;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
}

:deep(.cm-editor) {
  height: 100%;
  outline: none !important;
}

:deep(.cm-content) {
  padding: 20px 0;
}

:deep(.cm-gutters) {
  background-color: #282c34 !important;
  border-right: 1px solid #3e4451 !important;
  color: #abb2bf !important;
}

.refinement-bar {
  position: absolute;
  top: 16px;
  left: 50%;
  width: 90%;
  max-width: 440px;
  background: var(--surface-soft);
  border: 1px solid var(--accent-soft);
  border-radius: 20px;
  display: flex;
  padding: 4px 14px;
  box-shadow: 0 12px 40px rgba(0,0,0,0.5);
  z-index: 20;
}

.pdf-viewer {
  flex: 1;
  background: #525659;
}

.pdf-viewer iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.empty-preview {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-accent);
  color: var(--muted);
  text-align: center;
  padding: 40px;
}

.placeholder-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  max-width: 300px;
}

.placeholder-content h3 {
  font-size: 1rem;
  color: var(--ink);
  margin: 0;
}

.error-console {
  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  max-height: 30%;
  background: #1e1e1e;
  border-top: 1px solid var(--warning);
  display: flex;
  flex-direction: column;
  z-index: 50;
}

.console-header {
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  background: rgba(248, 81, 73, 0.1);
  border-bottom: 1px solid rgba(248, 81, 73, 0.2);
}

.error-logs {
  flex: 1;
  margin: 0;
  padding: 12px;
  overflow-y: auto;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  color: #f85149;
  line-height: 1.5;
  white-space: pre-wrap;
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background: rgba(13, 17, 23, 0.85);
  backdrop-filter: blur(8px);
  z-index: 10000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.spinner {
  color: var(--accent);
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@media (max-width: 1024px) {
  .workspace-sidebar {
    width: 200px;
  }
}

@media (max-width: 768px) {
  .split-pane {
    flex-direction: column;
  }
  
  .workspace-sidebar {
    width: 100%;
    height: 200px;
    border-right: none;
    border-bottom: 1px solid var(--line);
  }
  
  .editor-section {
    border-right: none;
    border-bottom: 1px solid var(--line);
  }
}
</style>
