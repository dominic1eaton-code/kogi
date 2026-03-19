import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioNotebookComponent } from './portfolio-notebook.component';

describe('PortfolioNotebookComponent', () => {
  let component: PortfolioNotebookComponent;
  let fixture: ComponentFixture<PortfolioNotebookComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioNotebookComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioNotebookComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
