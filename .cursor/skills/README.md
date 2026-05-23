# Cursor Skills

Bu dizindeki skill'ler [awesome-cursor-skills](https://github.com/spencerpauly/awesome-cursor-skills) deposundan yüklenmiştir.

Cursor agent bu dosyaları otomatik keşfeder. Her skill bir klasörde `SKILL.md` içerir:

```
.cursor/skills/<skill-name>/SKILL.md
```

## Kaynak

- Repo: https://github.com/spencerpauly/awesome-cursor-skills
- Lisans: MIT (kaynak repo ile aynı)

## Güncelleme

```bash
git clone --depth 1 https://github.com/spencerpauly/awesome-cursor-skills.git /tmp/awesome-cursor-skills
for dir in /tmp/awesome-cursor-skills/resources/*/; do
  skill=$(basename "$dir")
  mkdir -p ".cursor/skills/$skill"
  cp "$dir/SKILL.md" ".cursor/skills/$skill/"
done
```
