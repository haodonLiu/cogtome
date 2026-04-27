# COGTOME WebUI 组件库设计

## 目标

建立统一的 `ui/` 组件库，消除 CSS 与内联 styles 的混乱，提高维护效率和一致性。

## 组件清单

| 组件 | 用途 | CSS 变量依赖 |
|------|------|-------------|
| `Button` | 按钮 | `--accent`, `--accent-hover`, `--accent-light`, `--danger`, `--bg-card`, `--border`, `--text-secondary`, `--radius-md` |
| `Input` | 文本输入 | `--bg-page`, `--border`, `--accent`, `--text-primary`, `--text-secondary`, `--radius-sm` |
| `Textarea` | 多行输入 | 同 Input |
| `Select` | 下拉选择 | 同 Input |
| `Badge` | 标签/徽章 | `--accent`, `--accent-light`, `--danger`, `--warning`, `--success` |
| `Card` | 卡片容器 | `--bg-card`, `--border`, `--radius-lg`, `--shadow-sm` |
| `Modal` | 模态框 | `--bg-card`, `--panel-bg`, `--accent`, `--radius-lg` |
| `Spinner` | 加载指示 | `--accent`, `--border` |
| `EmptyState` | 空状态 | `--text-secondary`, `--text-tertiary` |
| `ErrorBanner` | 错误提示 | `--danger`, `--danger-bg` |
| `FormField` | 字段包装 | `--text-secondary` |

## API 设计

### Button
```tsx
<Button
  variant="primary" | "secondary" | "ghost" | "danger"
  size="sm" | "md" | "lg"
  disabled?: boolean
  loading?: boolean
  onClick?: () => void
  children: ReactNode
/>
```

### Input
```tsx
<Input
  type?: "text" | "number" | "password" | "email" | "search"
  placeholder?: string
  value?: string
  onChange?: (e: ChangeEvent) => void
  disabled?: boolean
  error?: boolean
  mono?: boolean  // 等宽字体
/>
```

### Badge
```tsx
<Badge
  variant="accent" | "success" | "warning" | "danger" | "secondary"
  size="sm" | "md"
>
```

### Card
```tsx
<Card
  hoverable?: boolean  // 悬停效果
  onClick?: () => void
  padding?: "sm" | "md" | "lg"
>
```

### Modal
```tsx
<Modal
  open: boolean
  onClose: () => void
  title?: string
  footer?: ReactNode
  width?: string
>
```

### FormField
```tsx
<FormField
  label?: string
  error?: string
  hint?: string
>
  {children}
</FormField>
```

## 特殊场景处理

| 场景 | 解决方案 |
|------|----------|
| 图形编辑器内节点样式 | 保持现有 inline styles，节点组件不迁移 |
| 列表页内联 styles | 迁移到 Card, Badge, Spinner 等组件 |
| 编辑器内 KV-editor | 保留结构，仅迁移 Input 部分 |
| 复杂样式覆盖 | 组件透传 `style` 和 `className` props |
| 编辑器 Toolbar | 迁移到 Button 组件 |

## 迁移顺序

1. **Phase 1**: 创建 11 个基础组件
2. **Phase 2**: 迁移列表页 (StructureList, MotifList, UnitList)
3. **Phase 3**: 迁移编辑器页面
4. **Phase 4**: 清理废弃 CSS

## CSS 清理

移除以下旧样式（迁移完成后）：
- `.btn-primary`, `.btn-secondary`, `.btn-ghost`, `.btn-danger`
- `.input`, `.input.textarea`
- `.card` (内联styles替代)
- `.spinner`
- `.error-banner`
- 重复的 `.section-title`, `.template-*`

## 验证方法

1. 编译无错误：`npm run build`
2. 功能检查：列表页、编辑器、模态框
3. 视觉对比：按钮颜色橙色、字体粗体、组件样式一致
