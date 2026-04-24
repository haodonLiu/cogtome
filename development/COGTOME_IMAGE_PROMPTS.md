# COGTOME Logo - 优化后的生图 Prompt

## 风格设定

- **背景**: 白色背景 (`white background`)
- **色调**: 工业古铜 + Warhammer 40K 暗色调
  - 古铜色: `#CD7F32`, `#B87333`, `#8B4513`
  - 深钢色: `#2F3640`, `#1C1C1C`
- **风格**: 2D 卡通，干净线条，无渐变阴影
- **关键词**: `2D cartoon`, `clean flat lines`, `no shading`, `industrial bronze copper`

---

## 5 个版本 Prompt

### v1 - 齿轮在书上
```
2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Thick hardcover book lying flat, ornate mechanical gears mounted on top, gears meshed together, steampunk, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading
```

### v2 - 齿轮嵌入书角
```
2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Hardcover book with four corners embedding small gears, gear corners as metal fixtures, steampunk, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading
```

### v3 - 齿轮形状的书
```
2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Open book shaped like a gear, pages form cog teeth radiating outward, gear-book fusion, steampunk, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading
```

### v4 - 齿轮从书页飘出
```
2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Open book with gears floating out like magic symbols, various sizes levitating and radiating, mystical mechanical, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading
```

### v5 - 一半书一半齿轮
```
2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Half hardcover book left, half gear right, seamlessly merged, book spine + pages on left, gear teeth + frame on right, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading
```

---

## MiniMax CLI 生成命令

```bash
# 版本1
mmx image "2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Thick hardcover book lying flat, ornate mechanical gears mounted on top, gears meshed together, steampunk, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading" --aspect-ratio 1:1

# 版本2
mmx image "2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Hardcover book with four corners embedding small gears, gear corners as metal fixtures, steampunk, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading" --aspect-ratio 1:1

# 版本3
mmx image "2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Open book shaped like a gear, pages form cog teeth radiating outward, gear-book fusion, steampunk, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading" --aspect-ratio 1:1

# 版本4
mmx image "2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Open book with gears floating out like magic symbols, various sizes levitating and radiating, mystical mechanical, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading" --aspect-ratio 1:1

# 版本5
mmx image "2D cartoon, white bg, industrial bronze copper, Warhammer 40K tones. Half hardcover book left, half gear right, seamlessly merged, book spine + pages on left, gear teeth + frame on right, bronze/copper (#CD7F32, #B87333) + dark steel (#2F3640), clean flat lines, no shading" --aspect-ratio 1:1
```

---

## Prompt 优化心得

1. **先说媒介和背景**: `2D cartoon, white bg`
2. **色调放前面**: `industrial bronze copper, Warhammer 40K tones`
3. **主体描述要具体**: 书的状态、齿轮的位置关系
4. **风格锁定**: `steampunk, clean flat lines, no shading`
5. **颜色精确指定**: 十六进制颜色码比文字描述更准确
6. **宽高比**: `--aspect-ratio 1:1` 或 `16:9` 按需选择

