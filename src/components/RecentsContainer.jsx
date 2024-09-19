export const RecentsContainer = ({ className, recent }) => {
  className += " col-span-7 p-2";
  console.log(recent)
  return (
    <section className={className}>
      <div>
        <span className="text-2xl">Recent Files</span>
      </div>
      <div>
        {recent.length > 0 && (
          <ul className="results-list flex flex-col gap-3">
            {recent.map(([filePath], index) => (
              <li
                key={index}
                className="result-item flex flex-row gap-1 border-b py-1"
                onClick={() => openFile(filePath)}
              >
                <span className="file-path text-muted-foreground text-ellipsis">
                  {filePath}
                </span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </section>
  );
};
