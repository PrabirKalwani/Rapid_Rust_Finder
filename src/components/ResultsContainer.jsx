import { Button } from "@/components/ui/button";

export const ResultsContainer = ({
  className,
  results,
  loading,
  error,
  query,
  openFile
}) => {
  className += " col-span-7 flex flex-col gap-3 p-2";

  const getFileIcon = (filename) => {
    const extension = filename.split(".").pop().toLowerCase();
    switch (extension) {
      case "pdf":
        return "pdf.png";
      case "docx":
        return "docx.png";
      case "xlsx":
        return "xlsx.png";
      case "jpg":
      case "jpeg":
        return "image.png";
      case "png":
        return "image.png";
      default:
        return "default.png";
    }
  };

  return (
    <section className={className}>
      <div>
        <span className="text-2xl">Results</span>
      </div>
      {results.length === 0 && !loading && !error && query.trim() !== "" && (
        <p className="no-results">No results found</p>
      )}
      {results.length > 0 && (
        <ul className="results-list flex flex-col gap-3">
          {results.map(([highlightedFilename, filePath], index) => (
            <li
              key={index}
              className="result-item flex flex-row gap-1 border-b py-1"
              onClick={() => openFile(filePath)}
            >
              <img
                className="icon h-12"
                src={`/icons/${getFileIcon(highlightedFilename)}`}
                alt="file-icon"
              />
              <div className="flex flex-col gap-1 justify-start align-top">
                <span
                  className="filename text-md "
                  dangerouslySetInnerHTML={{ __html: highlightedFilename }}
                ></span>
                <span className="file-path text-muted-foreground text-ellipsis">
                  {filePath}
                </span>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
};
