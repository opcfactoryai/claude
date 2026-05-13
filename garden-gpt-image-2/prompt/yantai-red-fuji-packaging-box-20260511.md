# 烟台红富士苹果外包装箱 · 生成提示词

## 任务描述

设计一款烟台红富士苹果的外包装瓦楞纸箱效果图，箱体尺寸 60cm × 40cm × 20cm，画面以立体透视角度展示包装箱的正面（60×40cm）和侧面（40×20cm），让品牌信息、苹果主体、高山元素、营养卖点一目了然。

## 结构化 Prompt（推荐用于 GPT Image 2 / DALL·E 3）

```json
{
  "type": "水果外包装纸箱设计效果图",
  "goal": "生成一张可直接用于印刷提案的烟台红富士苹果外包装箱立体展示图，突出高山种植与营养卖点，风格喜庆大气",
  "subject": {
    "packaging_box": {
      "type": "瓦楞纸箱",
      "material": "高强瓦楞纸板，表面覆哑膜",
      "dimensions": "60cm × 40cm × 20cm（长 × 宽 × 高）",
      "view_angle": "3/4 透视角度，正面为主（60×40cm 面朝镜头），右侧面（40×20cm）可见，体现箱体厚度与立体感"
    }
  },
  "layout": {
    "front_face_60x40": {
      "background": "自上而下渐变：顶部为蓝天白云 + 远处高山果园轮廓（烟台栖霞牙山山脉意象），中部过渡到暖白/米黄色底，底部为深红/中国红底色",
      "top_section": {
        "brand_name": "烟台红富士",
        "brand_style": "粗宋体/标题书法体，烫金效果，字号最大，居中偏上",
        "subtitle": "高山种植 · 自然好果",
        "subtitle_style": "楷体/仿宋，金色描边，位于品牌名下方"
      },
      "center_section": {
        "hero_image": "一颗或多颗饱满红富士苹果实物图，果皮鲜红光亮、果形端正、带着水珠，新鲜欲滴，占据正面视觉重心",
        "mountain_element": "苹果后方淡淡浮现高山梯田果园剪影，标注'海拔 800m+'",
        "apple_characteristics": "苹果周围点缀绿叶、露珠，体现新鲜采摘"
      },
      "bottom_section": {
        "selling_points": [
          "脆甜多汁 · 营养健康",
          "高海拔昼夜温差大 · 糖分积累充足"
        ],
        "selling_point_style": "白色粗体字，红色底上排列，一目了然",
        "nutrition_icons": "三个圆形图标并排，分别标注：维生素C、膳食纤维、抗氧化物质，图标简洁卡通风格",
        "target_audience_badge": "一个柔和圆角徽章，文字：'老人 · 小孩 · 孕妇 安心食用'，金色边框暖白底"
      }
    },
    "side_face_40x20": {
      "content": "竖向排版，顶部小号品牌名 + 苹果 icon，中部营养成分表（简洁版）+ 产品规格信息（品名：红富士苹果 / 产地：山东烟台 / 净重：约XXkg / 储存方式：阴凉通风处），底部联系方式区域",
      "contact_section": {
        "phone": "联系电话：400-XXX-XXXX",
        "wechat_qr": "一个微信二维码占位框（白色方块内含'二维码'字样 + 微信图标）",
        "qrcode_label": "扫码了解更多"
      }
    }
  },
  "color_palette": {
    "primary": "中国红 #C41E27（喜庆、大气、水果行业通用）",
    "secondary": "金色 #D4A843（品牌名、边框、点缀）",
    "accent": "苹果红 #E8302A + 果叶绿 #4A8C3F",
    "background_transition": "天蓝 → 米白 → 中国红（自上而下）",
    "text_on_dark": "白色 / 米白",
    "text_on_light": "深灰 #333"
  },
  "typography": {
    "brand_font": "粗宋体 / 行楷书法体（庄重、有中国味）",
    "subtitle_font": "楷体 / 仿宋",
    "body_font": "黑体（卖点、营养信息）",
    "max_font_family": "≤ 3 种",
    "language": "中文为主，英文/拼音为辅"
  },
  "scene": {
    "environment": "产品摄影棚灯光环境，白色/浅灰背景地面有轻微倒影，纸箱放在木质纹理展台上，增强真实感",
    "lighting": "45° 主光 + 正面柔光补光 + 底部反光板，纸箱表面无反光过曝，印刷内容清晰可读",
    "shadows": "纸箱右下角自然投影，不突兀"
  },
  "style": {
    "rendering": "高分辨率商业产品摄影 + 包装设计 mockup，纸箱材质纹理可见，印刷色彩饱满，烫金有金属光泽",
    "overall_feel": "喜庆大气、高品质、值得信赖、有送礼档次，同时保留农产品天然质朴感"
  },
  "constraints": {
    "must_keep": [
      "箱体比例严格符合 60:40:20（长:宽:高）",
      "品牌名 '烟台红富士' 清晰可读，是画面最醒目的文字",
      "高山种植元素明确可见（山脉、海拔标注）",
      "苹果实物照片级真实感，红润光亮",
      "营养图标（维生素C / 膳食纤维 / 抗氧化）清晰",
      "老人小孩孕妇适用标识可见",
      "联系方式区域（电话 + 二维码占位）清晰",
      "整体配色 ≤ 4 种主色",
      "正面（60×40）和侧面（40×20）信息层次分明"
    ],
    "avoid": [
      "苹果用绿色/未成熟颜色（必须是红富士的鲜红色）",
      "箱体比例变形，看起来像正方体",
      "文字模糊、字体超过 3 种",
      "画面过于花哨、信息杂乱",
      "背景过于暗沉失去喜庆感",
      "出现其他品牌/品种苹果",
      "包装看起来廉价（快递纸箱感）",
      "烫金效果过于耀眼导致文字不可读"
    ]
  }
}
```

---

## 自然语言版 Prompt（可直接复制使用）

> **推荐直接用下面这段自然语言 prompt 丢进 GPT Image 2 / DALL·E 3：**

---

A high-resolution commercial product photography render of a corrugated cardboard packaging box for **Yantai Red Fuji apples**, shown in a 3/4 perspective angle. The box dimensions are 60cm wide × 40cm tall × 20cm deep, with the front face (60×40cm) prominently facing the camera and the right side face (40×20cm) visible at an angle.

**Front Face Design (60cm × 40cm):**
- Background transitions top to bottom: light blue sky with white clouds → misty mountain orchard silhouette (Yantai Qixia high-altitude apple orchards at 800m+) → warm cream/off-white middle → rich Chinese red (#C41E27) at the bottom
- Top section: Brand name "烟台红富士" in large, bold serif/calligraphy font with gold foil stamping effect, centered. Below it, subtitle "高山种植 · 自然好果" in elegant script font with gold outline
- Center section: Photorealistic glossy Red Fuji apples as the hero image — vibrant red skin, perfectly round shape, with water droplets, surrounded by fresh green leaves. Behind the apples, a subtle silhouette of terraced mountain orchards with a label "海拔 800m+"
- Bottom section on red background: White bold text selling points — "脆甜多汁 · 营养健康" and "高海拔昼夜温差大 · 糖分积累充足". Three circular nutrition icons in a row: Vitamin C, Dietary Fiber, Antioxidants (simple, clean icon style). A soft rounded badge with gold border reading "老人 · 小孩 · 孕妇 安心食用"

**Side Face Design (40cm × 20cm):**
- Vertical layout with small brand logo at top
- Middle: simplified nutrition facts table and product specs (Product: 红富士苹果 / Origin: 山东烟台 / Storage: 阴凉通风处)
- Bottom: Contact section with phone number "400-XXX-XXXX" and a white WeChat QR code placeholder box with QR icon, labeled "扫码了解更多"

**Lighting & Environment:**
Studio product photography lighting with 45° key light, soft fill light, and subtle bottom reflector. The box sits on a warm wooden display surface with a soft natural drop shadow. Matte lamination finish on the box surface, with realistic gold foil metallic sheen on the brand name. Box edges are crisp and well-defined, showing the corrugated cardboard texture subtly at the edges.

**Overall Style:**
Festive yet premium Chinese agricultural packaging aesthetic. Rich reds and golds, photorealistic apple rendering, clean information hierarchy. The design balances "high-end gift box quality" with "authentic farm-fresh produce" feel. No clutter, no more than 4 dominant colors, all text sharp and readable.

**Aspect ratio: 1:1 (square), high quality, highly detailed, commercial product photography style, 8K resolution.**

---

## 使用说明

1. **GPT Image 2 / DALL·E 3**：直接复制上面自然语言版 prompt 即可生成
2. **Midjourney**：去掉 JSON 结构，用自然语言版，追加 `--ar 1:1 --style raw --s 250 --v 6`
3. **如需调整**：可以单独要求生成正面平面展开图（die-cut layout），把上面 prompt 中的 view_angle 改为 flat layout
4. **二维码**：生成的是占位框，实际印刷时替换为真实二维码

---

## 生成参数建议

| 平台 | 推荐尺寸 | 其他参数 |
|------|---------|---------|
| GPT Image 2 | 1024×1024 | quality: high |
| DALL·E 3 | 1024×1024 | quality: hd |
| Midjourney | 1:1 | --style raw --s 250 |
