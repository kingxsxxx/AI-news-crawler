import type { Article } from "../../types";

type Category = "Tech" | "Research" | "Product" | "Industry" | "Fun" | string;

interface CategoryBadgeProps {
  category: Category;
}

export function CategoryBadge({ category }: CategoryBadgeProps): JSX.Element {
  const getBadgeClass = (cat: Category): string => {
    const lower = cat.toLowerCase();
    if (lower.includes("tech")) return "tech";
    if (lower.includes("research")) return "research";
    if (lower.includes("product")) return "product";
    if (lower.includes("industry")) return "industry";
    if (lower.includes("fun")) return "fun";
    return "tech"; // 默认
  };

  return (
    <span className={`category-badge ${getBadgeClass(category)}`}>
      {category}
    </span>
  );
}
