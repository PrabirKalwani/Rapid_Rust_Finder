export const ResultsContainer = ({
  className,
  results,
  loading,
  error,
  query,
  openFile,
  getFileIcon,
  selectedFile,
  setSelectedFile,
}) => {
  className += " px-4 py-6";
  
  let lastClickTime = Date.now();

  const handleClicks = (file) => {
    const now = Date.now();
    // const difference = now - lastClickTime;
    // console.log(difference)
    if (now - lastClickTime < 300) {
      // console.log("Double-click detected");
      openFile(file);
    } else {
      // console.log("Single-click detected");
      // Highlight selected file immediately
      setSelectedFile(file === selectedFile ? null : file);
    }
    
    lastClickTime = now;
  };

  return (
    <section className={className}>
      {results.length === 0 && !loading && !error && query.trim() !== "" && (
        <p className="text-muted-foreground">No files to show</p>
      )}
      {results.length > 0 && (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-2">
          {results.map((file, index) => (
            <div
              className={`flex flex-col items-center text-ellipsis p-2 gap-2 ${
                file === selectedFile ? "bg-secondary shadow-sm rounded-sm" : ""
              }`}
              key={index}
              onClick={() => handleClicks(file)}
            >
              <img
                src={`/icons/${getFileIcon(file.fileName)}`}
                alt={file.fileName}
                className="w-8 h-8 sm:w-12 sm:h-12 md:w-16 md:h-16 lg:w-20 lg:h-20 xl:w-24 xl:h-24"
              />
              <p className="truncate w-full text-center text-sm">
                {file.fileName.split(".")[0]}
              </p>
            </div>
          ))}
        </div>
      )}
    </section>
  );
};
