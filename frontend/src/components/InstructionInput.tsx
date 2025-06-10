import { useEffect, useState } from "react";
import svg from "../assets/settings.svg"
interface InstructionInputProps {
  value: string;
  setValue: React.Dispatch<React.SetStateAction<string>>;
}

export function InstructionInput({ value, setValue }: InstructionInputProps) {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const saved = localStorage.getItem("instruction");
    if (saved) {
      setValue(saved);
    }
  }, [setValue]);

  useEffect(() => {
    localStorage.setItem("instruction", value);
  }, [value]);

  return (
    <div className="instruction-container">
      <div className="instruction-header">
        <button type="button" onClick={() => setOpen(!open)} className="instruction-toggle-button" aria-label="Afficher les instructions">
          <img src={svg} className="instruction-icon" />
        </button>

        {open && (
          <textarea
            value={value}
            onChange={(e) => setValue(e.target.value)}
            rows={4}
            className="instruction-textarea"
            placeholder="Exemple : réponds comme un expert Rust, ne fais pas d'hypothèses hors code source"
          />
        )}
      </div>
    </div>
  );
}
