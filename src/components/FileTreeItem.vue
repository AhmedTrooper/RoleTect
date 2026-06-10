<script setup lang="ts">
import { 
  ChevronRight, 
  ChevronDown, 
  File, 
  Plus, 
  FolderPlus, 
  Trash2, 
  Star,
  FileCode,
  Pencil
} from '@lucide/vue';
import { Motion, AnimatePresence } from 'motion-v';

interface FileItem {
  name: string;
  path: string;
  isDir: boolean;
  children?: FileItem[];
  isOpen?: boolean;
}

defineProps<{
  item: FileItem;
  activeFilePath: string | null;
  mainFilePath?: string | null;
  isDiagram?: boolean;
  onToggle: (item: FileItem) => void;
  onSelect: (item: FileItem) => void;
  onSetMain?: (path: string) => void;
  onCreateFile: (parent: FileItem, ext?: string) => void;
  onCreateFolder: (parent: FileItem) => void;
  onDelete: (item: FileItem) => void;
  onRename?: (item: FileItem) => void;
}>();
</script>

<template>
  <div class="tree-item-wrapper">
    <div 
      class="tree-item" 
      :class="{ 
        active: activeFilePath === item.path, 
        'main-file': mainFilePath === item.path 
      }"
      @click="item.isDir ? onToggle(item) : onSelect(item)"
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
        <!-- Main File Star (Compiler Only) -->
        <button 
          v-if="!item.isDir && onSetMain" 
          @click.stop="onSetMain(item.path)" 
          :title="mainFilePath === item.path ? 'Main File' : 'Set as Main File'"
          class="tree-action-btn"
        >
          <Star 
            :size="14" 
            :fill="mainFilePath === item.path ? 'var(--accent)' : 'none'" 
            :color="mainFilePath === item.path ? 'var(--accent)' : 'currentColor'" 
          />
        </button>

        <!-- Creation Actions (Folders Only) -->
        <template v-if="item.isDir">
          <button v-if="!isDiagram" @click.stop="onCreateFile(item)" title="New File" class="tree-action-btn"><Plus :size="16" /></button>
          <template v-else>
            <button @click.stop="onCreateFile(item, '.mmd')" title="New Diagram" class="tree-action-btn"><Plus :size="16" /></button>
            <button @click.stop="onCreateFile(item, '.md')" title="New Markdown" class="tree-action-btn"><FileCode :size="16" /></button>
          </template>
          <button @click.stop="onCreateFolder(item)" title="New Folder" class="tree-action-btn"><FolderPlus :size="16" /></button>
        </template>

        <!-- Rename Action -->
        <button v-if="onRename" class="tree-action-btn item-rename" @click.stop="onRename(item)" title="Rename"><Pencil :size="13" /></button>

        <!-- Delete Action -->
        <button class="tree-action-btn item-delete" @click.stop="onDelete(item)" title="Delete"><Trash2 :size="16" /></button>
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
        <!-- Recursion happens here -->
        <FileTreeItem 
          v-for="child in item.children" 
          :key="child.path"
          :item="child"
          :active-file-path="activeFilePath"
          :main-file-path="mainFilePath"
          :is-diagram="isDiagram"
          :on-toggle="onToggle"
          :on-select="onSelect"
          :on-set-main="onSetMain"
          :on-create-file="onCreateFile"
          :on-create-folder="onCreateFolder"
          :on-delete="onDelete"
          :on-rename="onRename"
        />
      </Motion>
    </AnimatePresence>
  </div>
</template>

<style scoped>
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

.tree-item.main-file {
  font-weight: 700;
}

.tree-item.main-file .item-name {
  color: var(--accent);
}

.item-icon {
  display: flex;
  align-items: center;
  color: var(--muted);
  flex-shrink: 0;
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
  gap: 2px;
  opacity: 0;
  flex-shrink: 0;
}

.tree-item:hover .item-actions {
  opacity: 1;
}

.tree-action-btn {
  background: none;
  border: none;
  color: var(--muted);
  cursor: pointer;
  padding: 6px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.tree-action-btn:hover {
  background: var(--surface);
  color: var(--ink);
}

.tree-action-btn.item-delete:hover {
  background: rgba(248, 81, 73, 0.1);
  color: var(--warning);
}

.tree-children {
  padding-left: 12px;
  overflow: hidden;
}
</style>
